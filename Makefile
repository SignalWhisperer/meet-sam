build-%:
	cargo lambda build --release --bin $*
	cp target/lambda/$*/bootstrap ${ARTIFACTS_DIR}
