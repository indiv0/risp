.PHONY: all clean

STEPS = step0_repl

all: $(STEPS)

%: %.rs
	cargo build --release --bin $*
	cp target/release/$* $@

clean:
	cargo clean
	rm -f $(STEPS)
