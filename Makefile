out_files = pub/ download/ index.html cv.html

.PHONY: clean
clean:
	@echo cleaning the workspace.. && /usr/bin/rm -rf $(out_files)
