out_files = junkyard/ download/ index.html cv.html

.PHONY: clean
clean:
	@echo cleaning the workspace.. && /usr/bin/rm -rf $(out_files)
