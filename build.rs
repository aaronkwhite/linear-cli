fn main() {
    cynic_codegen::register_schema("linear")
        .from_sdl_file("schemas/linear.graphql")
        .unwrap()
        .as_default()
        .unwrap();
}
