.PHONY: all clean

STEPS = step0_repl step1_read_print

all: $(STEPS)

%: %.rs
	cargo build --release --bin $*
	cp target/release/$* $@

clean:
	cargo clean
	rm -f $(STEPS)
