import tomllib

with open("rust-toolchain.toml", "rb") as f:
    print(tomllib.load(f)["toolchain"]["channel"])
