.PHONY: run interactive

run:
	time cargo run

interactive:
	INTERACTIVE=true time cargo run; stty sane

i: interactive
