import rust from "@wasm-tool/rollup-plugin-rust";

export default {
  format: "es",
  input: {
    amazegen: "../../Cargo.toml",
  },
  plugins: [rust()],
};
