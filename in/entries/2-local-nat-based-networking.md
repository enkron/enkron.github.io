# setting up NAT based network for using with libvirt

The following steps are almost copy/paste from the [quide][1] and it
isn\'t pretending for content originality.
It serves for reproducing through the prism of my perception, i\'m very
grateful to the author for the detailed analysis of this topic.

The `libvirt` daemon creates default network which exploits virbr0
bridge interface. The network has limitations as the daemon
automatically inserts iptables rules for the particular 192.168.122.0/24
subnet.

[1]: https://jamielinux.com/docs/libvirt-networking-handbook/custom-nat-based-network.html

## disable default libvirt network

```bash
virsh net-destroy default
virsh net-autostart --disable default
virsh net-undefine --network default
```

## virtual bridge

### create bridge network interface

```bash
sudo ip link add name k8s-br0 type bridge
```

### enable stp (Spanning Tree Protocol) on a bridge

```bash
sudo ip link set k8s-br0 type bridge stp_state 1
```

### create virtual network interface

We need to attach virtual network interface `k8s-br0` to just created
bridge to provide a stabe MAC address for the interface (a bridge
inherits MAC address of a first interface connected to it thus it will
changing every time new VM is connected without the dummy interface).

**NOTE** In order to provide internet communication path for kvm vms
bridge should be attached to physical network interface. However it
won't work in case the interface's connection type is wireless.

### load `dummy` kernel module

```bash
sudo modprobe dummy
```

### create virtual interface

```bash
sudo ip link add k8s-br0-nic type dummy
```

Generate MAC address and assign it to the dummy network internet
controller.  MAC address has to be in format that `libvirt` expects
(52:54:00:XX:XX:XX for KVM based virtualization)

```bash
MAC=$(hexdump -vn3 -e '/3 "52:54:00"' -e '/1 ":%02x"' -e '"\n"' /dev/urandom)
sudo ip link set dev k8s-br0-nic address $MAC
```

### add the virtual interface to the bridge

```bash
sudo ip link set k8s-br0-nic master k8s-br0
```

### verify the interface was added to the bridge

```bash
ip link show master k8s-br0
```

### assign static ip address to the bridge

The private subnet will be 192.168.100.0/24

```bash
sudo ip addr add 192.168.100.1/24 dev k8s-br0 broadcast 192.168.100.255
```

### turn on the network interfaces

```bash
sudo ip link set k8s-br0 up
sudo ip link set k8s-br0-nic up
```

## NAT

### allow packets forwarding

```bash
echo "net.ipv4.ip_forward=1" >> /etc/sysctl.conf
echo "net.ipv4.conf.all.forwarding=1" >> /etc/sysctl.conf
sysctl -p
```

### ip masquerading & packets forwarding rules in iptables

```bash
cat << EOF > ip-masquerade.txt
*mangle
:PREROUTING ACCEPT [0:0]
:INPUT ACCEPT [0:0]
:FORWARD ACCEPT [0:0]
:OUTPUT ACCEPT [0:0]
:POSTROUTING ACCEPT [0:0]
# DHCP packets sent to VMs have no checksum (due to a longstanding bug).
-A POSTROUTING -o k8s-br0 -p udp -m udp --dport 68 -j CHECKSUM --checksum-fill
COMMIT

*nat
:PREROUTING ACCEPT [0:0]
:OUTPUT ACCEPT [0:0]
:POSTROUTING ACCEPT [0:0]
# Do not masquerade to these reserved address blocks.
-A POSTROUTING -s 192.168.100.0/24 -d 224.0.0.0/24 -j RETURN
-A POSTROUTING -s 192.168.100.0/24 -d 255.255.255.255/32 -j RETURN
# Masquerade all packets going from VMs to the LAN/Internet.
-A POSTROUTING -s 192.168.100.0/24 ! -d 192.168.100.0/24 -p tcp -j MASQUERADE --to-ports 1024-65535
-A POSTROUTING -s 192.168.100.0/24 ! -d 192.168.100.0/24 -p udp -j MASQUERADE --to-ports 1024-65535
-A POSTROUTING -s 192.168.100.0/24 ! -d 192.168.100.0/24 -j MASQUERADE
COMMIT

*filter
:INPUT ACCEPT [0:0]
:FORWARD ACCEPT [0:0]
:OUTPUT ACCEPT [0:0]
# Allow basic INPUT traffic.
-A INPUT -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
-A INPUT -i lo -j ACCEPT
-A INPUT -p icmp --icmp-type 8 -m conntrack --ctstate NEW -j ACCEPT
# Accept SSH connections.
-A INPUT -p tcp -m tcp --syn -m conntrack --ctstate NEW --dport 22 -j ACCEPT
# Accept DNS (port 53) and DHCP (port 67) packets from VMs.
-A INPUT -i k8s-br0 -p udp -m udp -m multiport --dports 53,67 -j ACCEPT
-A INPUT -i k8s-br0 -p tcp -m tcp -m multiport --dports 53,67 -j ACCEPT
# Reject everything else.
-A INPUT -m conntrack --ctstate INVALID -j DROP
-A INPUT -p tcp -m tcp -j REJECT --reject-with tcp-reset
-A INPUT -j REJECT --reject-with icmp-port-unreachable

# Allow established traffic to the private subnet.
-A FORWARD -d 192.168.100.0/24 -o k8s-br0 -m conntrack --ctstate RELATED,ESTABLISHED -j ACCEPT
# Allow outbound traffic from the private subnet.
-A FORWARD -s 192.168.100.0/24 -i k8s-br0 -j ACCEPT
# Allow traffic between virtual machines.
-A FORWARD -i k8s-br0 -o k8s-br0 -j ACCEPT
# Reject everything else.
-A FORWARD -i k8s-br0 -j REJECT --reject-with icmp-port-unreachable
-A FORWARD -o k8s-br0 -j REJECT --reject-with icmp-port-unreachable
COMMIT
EOF
```

### modify iptables

```bash
sudo iptables-restore -n -v < ip-masquerade.txt
```

## dnsmasq

The `libvirt` server runs its own `dnsmasq` instance to assign ip addr
via dhcp to each vm.

### create k8s-br0 interface specific files & dirs needed to run dnsmasq

```bash
mkdir -p /var/lib/dnsmasq/k8s-br0
touch /var/lib/dnsmasq/k8s-br0/hostsfile
touch /var/lib/dnsmasq/k8s-br0/leases
```

`hostsfile` - is using to assign a specific ip & mac addresses to a vm,
`leases` - contains information regarding dhcp leases for vms (eg. ip/mac/uuid).

### create dnsmasq configuration file

```bash
cat << EOF > /var/lib/dnsmasq/k8s-br0/dnsmasq.conf
# Only bind to the virtual bridge. This avoids conflicts with other
running
# dnsmasq instances.
except-interface=lo
interface=k8s-br0
bind-dynamic

# IPv4 addresses to offer to VMs. This should match the chosen subnet.
dhcp-range=192.168.100.2,192.168.100.254

# Set this to at least the total number of addresses in DHCP-enabled
subnets.
dhcp-lease-max=1000

# File to write DHCP lease information to.
dhcp-leasefile=/var/lib/dnsmasq/k8s-br0/leases

# File to read DHCP host information from.
dhcp-hostsfile=/var/lib/dnsmasq/k8s-br0/hostsfile

# Avoid problems with old or broken clients.
dhcp-no-override

# https://www.redhat.com/archives/libvir-list/2010-March/msg00038.html
strict-order
EOF
```

### run dnsmasq instance

```bash
sudo dnsmasq \
    --conf-file=/var/lib/dnsmasq/k8s-br0/dnsmasq.conf \
    --pid-file=/var/run/dnsmasq/k8s-br0.pid
```
