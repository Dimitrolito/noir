use super::{PROOFS_DIR, PROOF_EXT, VERIFIER_INPUT_FILE};
use crate::resolver::Resolver;
use crate::write_stderr;
use clap::ArgMatches;
use noir_field::FieldElement;
use noirc_abi::{input_parser::InputValue, Abi};
use std::{collections::BTreeMap, path::Path};

pub(crate) fn run(args: ArgMatches) {
    let proof_name = args
        .subcommand_matches("verify")
        .unwrap()
        .value_of("proof")
        .unwrap();
    let mut proof_path = std::path::PathBuf::new();
    proof_path.push(Path::new(PROOFS_DIR));

    proof_path.push(Path::new(proof_name));
    proof_path.set_extension(PROOF_EXT);

    let result = verify(proof_name);
    println!("Proof verified : {}\n", result);
}

fn verify(proof_name: &str) -> bool {
    let curr_dir = std::env::current_dir().unwrap();
    let (mut driver, backend_ptr) = Resolver::resolve_root_config(&curr_dir);
    let compiled_program = driver.into_compiled_program(backend_ptr);

    let mut proof_path = curr_dir;
    proof_path.push(Path::new("proofs"));
    proof_path.push(Path::new(proof_name));
    proof_path.set_extension(PROOF_EXT);

    let public_abi = compiled_program.abi.clone().unwrap().public_abi();
    let num_params = public_abi.num_parameters();

    let mut public_inputs = BTreeMap::new();
    if num_params != 0 {
        let curr_dir = std::env::current_dir().unwrap();
        public_inputs = noirc_abi::input_parser::Format::Toml.parse(curr_dir, VERIFIER_INPUT_FILE);
    }

    if num_params != public_inputs.len() {
        panic!(
            "Expected {} number of values, but got {} number of values",
            num_params,
            public_inputs.len()
        )
    }

    let public_inputs = process_abi_with_verifier_input(public_abi, public_inputs);

    // XXX: Instead of unwrap, return a PathNotValidError
    let proof_hex: Vec<_> = std::fs::read(proof_path).unwrap();
    // XXX: Instead of unwrap, return a ProofNotValidError
    let proof = hex::decode(proof_hex).unwrap();

    backend_ptr
        .backend()
        .verify_from_cs(&proof, public_inputs, compiled_program.circuit)
}

fn process_abi_with_verifier_input(
    abi: Abi,
    pi_map: BTreeMap<String, InputValue>,
) -> Vec<FieldElement> {
    let mut public_inputs = Vec::with_capacity(pi_map.len());

    for (param_name, param_type) in abi.parameters.into_iter() {
        let value = pi_map
            .get(&param_name)
            .expect(&format!(
                "ABI expects the parameter `{}`, but this was not found",
                param_name
            ))
            .clone();

        if !value.matches_abi(param_type) {
            write_stderr(&format!("The parameters in the main do not match the parameters in the {}.toml file. \n Please check `{}` parameter. ", VERIFIER_INPUT_FILE,param_name))
        }

        match value {
            InputValue::Field(elem) => public_inputs.push(elem),
            InputValue::Vec(vec_elem) => public_inputs.extend(vec_elem),
        }
    }

    public_inputs
}
