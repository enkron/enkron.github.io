.PHONY: clean

out_files = pub/ download/ index.html

clean:
	@echo cleaning the workspace.. && /usr/bin/rm -rf $(out_files)
