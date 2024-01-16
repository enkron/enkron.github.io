# setting up NAT based network (WIP)

0. disable default libvirt network

```bash
virsh net-destroy default
virsh net-autostart --disable default
virsh net-undefine --network default
```

## virtual bridge

0. create bridge network interface

```bash
sudo ip link add name k8s-br0 type bridge
```

Enable stp (Spanning Tree Protocol) on a bridge

```bash
sudo ip link set k8s-br0 type bridge stp_state 1
```

1. create virtual network interface

We need to attach virtual network interface `k8s-br0` to just created
bridge to provide a stabe MAC address for the interface (a bridge
inherits MAC address of a first interface connected to it thus it will
changing every time new VM is connected without connected interface).

**NOTE** In order to provide internet communication path for kvm vms
bridge should be attached to physical network interface. However it
won't work in case the interface's connection type is wireless.

Load `dummy` kernel module

```bash
sudo modprobe dummy
```

Create virtual interface

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

Add the virtual interface to the bridge

```bash
sudo ip link set k8s-br0-nic master k8s-br0
```

Verify the interface was added to the bridge

```bash
ip link show master k8s-br0
```

Assign static ip address to the bridge

The private subnet will be 192.168.100.0/24

```bash
sudo ip addr add 192.168.100.1/24 dev k8s-br0 broadcast 192.168.100.255
```

## NAT

Allow packets forwarding

```bash
echo "net.ipv4.ip_forward=1" >> /etc/sysctl.conf
echo "net.ipv4.conf.all.forwarding=1" >> /etc/sysctl.conf
sysctl -p
```

ip masquerading & packets forwarding rules in iptables

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

...
