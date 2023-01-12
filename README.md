# Agent2JSI_FASTQ_Converter

A command line tool to convert Agilent Agent Trimmed FAST files compatible for JSI input to use MBC.

Agilent dual MBC reads contain MBC and 1-2 dark bases in between the insert and UMI. The Agent Trimmer Tool from Agilent Technologies, trims the MBS and dark bases from 5' of R1 and R2. By default it add those infromation as comment to the read header in fastq files.

This tool takes reads the comments from the header and rebuilds the UMI+Read structure without the dark bases.

The JSI Sequence Pilot user interface lets you to identify only UMI as 5' or 3'. So the new files should be ready to be used in JSI.


USAGE:
    Agent2JSI_converter[.exe] --fastq <FILE> --pair <MODE>

OPTIONS:
    -f, --fastq <FILE>    Fastq files fastq.gz
    -h, --help            Print help information
    -p, --pair <MODE>     Read pair for fastq files [possible values: R1, R2]
    -V, --version         Print version information

### GNU GPL v3
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](https://www.gnu.org/licenses/gpl-3.0)    
