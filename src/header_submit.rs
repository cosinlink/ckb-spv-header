use crate::header_verifier::{HeaderVerifier, HeaderProvider};
use ckb_chain_spec::consensus::Consensus;
use ckb_types::{
    core::HeaderView,
    packed::{self, Byte32},
    prelude::*,
};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};


pub struct HeaderProviderWrapper<'a> {
    pub store: &'a ChainStore,
}

impl<'a> HeaderProvider for HeaderProviderWrapper<'a> {
    fn get_header(&self, hash: packed::Byte32) -> Option<HeaderView> {
        self.store.get_header(hash).expect("store should be OK")
    }
}

#[derive(Clone, Debug)]
pub struct ChainStore {
    pub consensus: Consensus,
    pub headers: Vec<HeaderView>
}

impl ChainStore {
    pub fn insert_header(&mut self, header: HeaderView) -> Result<(), String> {
        Ok(self.headers.push(header))
    }

    pub fn get_header(&self, block_hash: packed::Byte32) -> Result<Option<HeaderView>, String> {
        Ok(self.headers
            .iter()
            .find(|header| header.hash() == block_hash)
            .and_then(|item: &HeaderView| Some(item.clone()) )
        )
    }

    pub fn tip(&self) -> Result<Option<HeaderView>, String> {
        if self.headers.len() == 0 {
            return Err("headers empty".to_owned())
        }

        Ok(self.headers.get(self.headers.len() - 1)
               .and_then(|item: &HeaderView| Some(item.clone()))
        )

    }

    pub fn submit_headers(&mut self, headers: Vec<HeaderView>) {
        let len = headers.len();
        if len == 0 {
            return
        }

        let clone_store = self.clone();
        let header_provider = HeaderProviderWrapper { store: &clone_store };
        let header_verifier = HeaderVerifier::new(&clone_store.consensus, &header_provider);
        for header in headers {
            match header_verifier.verify(&header) {
                Ok(_) => self
                    .insert_header(header)
                    .expect("store should be OK"),
                Err(err) => {
                    return;
                }
            }
        }
    }
}