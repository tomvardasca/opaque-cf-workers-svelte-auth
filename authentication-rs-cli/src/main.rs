use authentication_rs_lib::AuthenticationOpaque;
use opaque_ke::{
    ciphersuite::CipherSuite, rand::rngs::OsRng
};

fn main() {
    println!("Will generate new server  key 🔐 :");
    let mut rng = OsRng;
    let server_kp = AuthenticationOpaque::generate_random_keypair(&mut rng);
    println!("New key: {}", base64::encode( server_kp.private().to_arr().to_vec()));
}
