fn main() {
    tonic_build::compile_protos("protos/helloworld/helloworld.proto").unwrap();
}
