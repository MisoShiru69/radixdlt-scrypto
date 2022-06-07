use clap::Parser;
use colored::*;
use rand::Rng;
use scrypto::call_data;
use scrypto::prelude::*;

use crate::resim::*;

/// Create an account
#[derive(Parser, Debug)]
pub struct NewAccount {
    /// Output a transaction manifest without execution
    #[clap(short, long)]
    manifest: Option<PathBuf>,

    /// Turn on tracing
    #[clap(short, long)]
    trace: bool,
}

impl NewAccount {
    pub fn run<O: std::io::Write>(&self, out: &mut O) -> Result<(), Error> {
        let secret = rand::thread_rng().gen::<[u8; 32]>();
        let private_key = EcdsaPrivateKey::from_bytes(&secret).unwrap();
        let public_key = private_key.public_key();
        let auth_address = NonFungibleAddress::from_public_key(&public_key);
        let withdraw_auth = rule!(require(auth_address));
        let manifest = ManifestBuilder::new()
            .call_method(SYSTEM_COMPONENT, call_data!(free_xrd()))
            .take_from_worktop(RADIX_TOKEN, |builder, bucket_id| {
                builder.new_account_with_resource(&withdraw_auth, bucket_id)
            })
            .build();

        let receipt = handle_manifest(
            manifest,
            &Some("".to_string()), // explicit empty signer public keys
            &self.manifest,
            self.trace,
            false,
            out,
        )?;

        if let Some(receipt) = receipt {
            let account = receipt.new_component_addresses[0];
            writeln!(out, "A new account has been created!").map_err(Error::IOError)?;
            writeln!(
                out,
                "Account component address: {}",
                account.to_string().green()
            )
            .map_err(Error::IOError)?;
            writeln!(out, "Public key: {}", public_key.to_string().green())
                .map_err(Error::IOError)?;
            writeln!(
                out,
                "Private key: {}",
                hex::encode(private_key.to_bytes()).green()
            )
            .map_err(Error::IOError)?;

            if get_configs()?.is_none() {
                writeln!(
                    out,
                    "No configuration found on system. will use the above account as default."
                )
                .map_err(Error::IOError)?;
                set_configs(&Configs {
                    default_account: account,
                    default_private_key: private_key.to_bytes(),
                    nonce: 0,
                })?;
            }
        } else {
            writeln!(out, "A manifest has been produced for the following key pair. To complete account creation, you will need to run the manifest!").map_err(Error::IOError)?;
            writeln!(out, "Public key: {}", public_key.to_string().green())
                .map_err(Error::IOError)?;
            writeln!(
                out,
                "Private key: {}",
                hex::encode(private_key.to_bytes()).green()
            )
            .map_err(Error::IOError)?;
        }
        Ok(())
    }
}
