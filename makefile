build:
	cd client; cargo build --release;
	cd installer; cargo build --release;
	cd updater; cargo build --release;
	cd server; cargo build --release;