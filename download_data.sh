#!/bin/bash
cd "$(dirname "$0")"
mkdir data
cd data
wget https://qoaformat.org/samples/qoa_test_samples_2023_02_18.zip
unzip qoa_test_samples_2023_02_18.zip
