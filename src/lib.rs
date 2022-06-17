#[macro_use]
extern crate lazy_static;

use std::io::{Read, BufReader};
use lzma_rs::xz_decompress;
use tar::Archive;

#[derive(Debug)]
pub enum DecompressionError {
    FileReadError,
    XzDecompressionError
}
pub fn untar_xz<R>(input: &mut BufReader<R>) -> Result<Vec<Vec<u8>>,  DecompressionError> 
where R: Read
{
    // first decompress and then use archive output
    let mut decompressed_input: Vec<u8> = Vec::new();
    if let Ok(()) = xz_decompress(input, &mut decompressed_input) {
        let mut tar_file_from_decompress_output = Archive::new(decompressed_input.as_slice());
        let mut obj_of_files: Vec<Vec<u8>> = Vec::new();
        for file in tar_file_from_decompress_output.entries().unwrap() {
            let mut file = file.unwrap();
            // println!("{:?}", file.header().path().unwrap());
            let mut file_vec = Vec::new();
            match file.read_to_end(&mut file_vec){
                Ok(_) => {},
                Err(_) => {return Err(DecompressionError::FileReadError)},
            };
            obj_of_files.push(file_vec);
        }
        Ok(obj_of_files)
    } else {
        Err(DecompressionError::XzDecompressionError)
    }
}

#[cfg(test)]
mod tests {
    

    use hex::encode;
    use sha3::{Digest, Sha3_384};
    use std::fs::File;
    use super::untar_xz;
    use std::path::Path;
    use std::env;
    lazy_static! {
    // ("sha3-384 sum, file in test-fixtures")
        static ref TAR_TEST_FILE: (&'static str, &'static str) = ("f14ff1d579be126f667a8a4719b3a2883aa41033495fec04fd0002626b94c9031c4ffd2c104884ba43bffedd6cd75f14",  "from_tar_command.tar.lzma");
        static ref HELLO_WORLD_FILE: (&'static str, &'static str) = ("28fc308d4d5c1ef9e60acedb13c3a1fcf7266560602c639000580ae3541dea5ce78a685de897e96b65a0fc15515c3780",  "hello_world.txt");
        static ref RAND_BYTES_FILE_1_FILE: (&'static str, &'static str) = ("05bb489111714cb379b6312123a778897a5616752f5cd4f12d97f4338e7599c21b08c8c3312a3addef10033cbc7f207f",  "rand_bytes_file_1");
        static ref RAND_BYTES_FILE_2_FILE: (&'static str, &'static str)= ("425da8b8b66fc8940cfebc00de20a1785bf37e453e925f3f819359bb43e56920c991ab0cfdc5a2f09611e3682e07f88e",  "rand_bytes_file_2");
    }

    // tar -c --lzma -f from_tar_command.tar.lzma rand_bytes_file_1 rand_bytes_file_2 hello_world.txt 
    #[test]
    fn test_untar_xz() {
        let hash_results = [RAND_BYTES_FILE_1_FILE.0, RAND_BYTES_FILE_2_FILE.0, HELLO_WORLD_FILE.0];
        let tar_file_path = format!("{}{}{}", env::current_dir().unwrap().display(), "/test-fixtures/", TAR_TEST_FILE.1);
        let tar_path = Path::new(tar_file_path.as_str());
        let file_ptr = File::open(tar_path).unwrap();
        let mut fs = std::io::BufReader::new(file_ptr);
        let files = untar_xz(&mut fs).unwrap();
        let mut hasher = Sha3_384::new();

        hasher.update(files[0].as_slice());
        let result = encode(hasher.finalize_reset());
        assert_eq!(result.as_str(), hash_results[0], "failed equality check");

        hasher.update(files[1].as_slice());
        let result = encode(hasher.finalize_reset());
        assert_eq!(result.as_str(), hash_results[1], "failed equality check");

        hasher.update(files[2].as_slice());
        let result = encode(hasher.finalize_reset());
        assert_eq!(result.as_str(), hash_results[2], "failed equality check");

    }
}
