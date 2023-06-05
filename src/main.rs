use anyhow::Context;
use rayon::prelude::*;
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

fn main() -> anyhow::Result<ExitCode> {
    if std::env::args().count() != 2 {
        eprintln!("usage: DATADIR");
        return Ok(ExitCode::FAILURE);
    }
    let data_dir = std::env::args().nth(1).unwrap();
    let glob = data_dir.to_string() + "/**/*.qoa.wav";
    let gold_wav_paths: Vec<_> = glob::glob(&glob).unwrap().collect();
    let results: Vec<_> = gold_wav_paths
        .into_par_iter()
        .map(|gold_wav_path| -> anyhow::Result<bool> {
            let gold_wav_path = gold_wav_path?;
            let qoa_path = get_qoa_path(&gold_wav_path);
            compare(&gold_wav_path, &qoa_path)
        })
        .collect();

    let mut success = 0;
    let mut failures = 0;
    let mut errors = 0;
    for result in results {
        match result {
            Ok(true) => success += 1,
            Ok(false) => failures += 1,
            Err(e) => {
                errors += 1;
                println!("error: {:?}", e)
            }
        }
    }

    println!("success: {success}");
    println!("failures: {failures}");
    println!("errors: {errors}");

    if success > 20 && failures == 0 && errors == 0 {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

fn compare(gold_wav: &Path, qoa: &Path) -> anyhow::Result<bool> {
    println!(
        "Comparing: {}",
        gold_wav.file_name().unwrap().to_str().unwrap()
    );

    let wav_decoder = hound::WavReader::open(gold_wav)
        .with_context(|| format!("open WavReader: {}", gold_wav.display()))?;
    let mut wav_iterator = wav_decoder.into_samples();

    let qoa_decoder = qoaudio::QoaDecoder::open(qoa)
        .with_context(|| format!("open QoaDecoder: {}", qoa.display()))?;
    let mut qoa_iterator =
        qoa_decoder.filter(|i| !matches!(i, Ok(qoaudio::QoaItem::FrameHeader(_))));

    let mut sample_idx = 0;
    loop {
        let next_wav = wav_iterator.next();
        let next_qoa = qoa_iterator.next();
        match (next_wav, next_qoa) {
            (None, None) => break,
            (Some(Ok(wav_sample)), Some(Ok(qoaudio::QoaItem::Sample(qoa_sample)))) => {
                if qoa_sample != wav_sample {
                    println!(
                        "Samples not equal at sample index {}. qoa_wav: {} qoaudio: {}",
                        sample_idx, wav_sample, qoa_sample
                    );
                    return Ok(false);
                }
            }
            (wav, qoa) => {
                println!(
                    "Samples not equal at sample index {}. qoa_wav: {:?} qoaudio: {:?}",
                    sample_idx, wav, qoa
                );
                return Ok(false);
            }
        }
        sample_idx += 1;
    }

    Ok(true)
}

fn get_qoa_path(path: &Path) -> PathBuf {
    let string = path.as_os_str().to_str().unwrap().replace(".wav", "");
    let string = string.replace("/qoa_wav/", "/qoa/");
    string.into()
}
