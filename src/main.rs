//use flate2::read::GzDecoder; // flate2 bug on concatated files while decomression, commented!
/*
use rust_htslib::bgzf::Reader; // omit flate to once success full
*/
use flate2::write::GzEncoder;
use flate2::read::MultiGzDecoder;
use flate2::Compression;
use std::io;
use std::io::prelude::*;
use fastq::Parser; // not used directly but keep here for future reference
use fastq::Record;
use std::str;
use std::path::Path;
use std::path::PathBuf;
use clap::{arg, command, value_parser, ArgAction, Command};

// Capacity 
const CAPACITY: usize = 10240; // will be used on flate2::GzDecoder,commented now!
fn check_inputfiles(inputfilename:&PathBuf)->Result<PathBuf,io::Error>{
    //check if path exists
    if ! inputfilename.exists() { return(Err(std::io::Error::new(io::ErrorKind::NotFound,"File not found")))  }
    // check if parent directory writable !!!
    let inputdirectory=inputfilename.parent().unwrap();
    // give this error while creating output file
    // if ! std::fs::metadata(inputfilename).unwrap().permissions().readonly() { return(Err(std::io::Error::new(io::ErrorKind::PermissionDenied,"Can't write to directory")))       }
    let infilename = inputfilename.file_name().unwrap();
    let infilestem=inputfilename.file_stem().unwrap();
    let infilestem2=PathBuf::from(infilestem).file_stem().unwrap().to_owned();
    let mut outfilename=String::new();
    outfilename.push_str(infilestem2.to_str().unwrap());
    outfilename.push_str("_renamed.fastq.gz");
    let mut outputbuffer=PathBuf::from(&inputdirectory);
    outputbuffer.push(&outfilename);
    //check if filename contains "gz"
    match infilename.to_str() {
        Some(s) => {
                        if ! s.ends_with("gz"){ return(Err(std::io::Error::new(io::ErrorKind::InvalidInput,"Not Gzip file"))) }
                },
        _=>{},
    }
    Ok(outputbuffer)
    
}
fn convert_fastq(inputfilename:&PathBuf , outputfilename:&PathBuf ,pair:&str) ->Result<(),io::Error>{
    // Input values:
    /**/
    // While using gzip decoder from flate2 
    //let in_filename = "input.fastq.gz";
    let in_fh = std::fs::File::open(inputfilename).unwrap();
    let in_gz = MultiGzDecoder::new(in_fh);
    let in_buf = io::BufReader::with_capacity(CAPACITY, in_gz);
    /*
    // While using rust_htslib
    let in_buf:Reader ;
    // * /
    match Reader::from_path(inputfilename){
        Ok(buf) => in_buf = buf,
        Err(e) => {
            // will be usign flate2 in the future, so don't try to convert error types
            return Err(std::io::Error::new(io::ErrorKind::InvalidInput,format!("{:?}",e)))
        },
    }
    */
    //Output values
    //let out_filename = "output.fastq.gz";
    let out_fh = std::fs::File::create(outputfilename)?;
    let out_gz = GzEncoder::new(out_fh, Compression::default());
    let mut out_buf = io::BufWriter::new(out_gz);
    
    // Read using the fastq::Parser
    let parser = fastq::Parser::new(in_buf);
    let mut readcount=0;
    parser.each( |record| {
        readcount+=1;
        let mut c_record = record.to_owned_record();
        // Update to the header
        let header:&str = str::from_utf8(&c_record.head).unwrap();
        //check if heder contains "/"
        let headerpresplit:Vec<&str>= header.split("\t").collect();
        // Assign Barcode, MBC Seqs and MBC Qualities.
        let mut BCZ=String::new();
        let mut RXZ=String::new();
        let mut QXZ =String::new();
        for comment in &headerpresplit{
            if comment.starts_with("BC:Z:"){ BCZ.push_str(comment); }
            if comment.starts_with("RX:Z:"){ RXZ.push_str(comment); }
            if comment.starts_with("QX:Z:"){ QXZ.push_str(comment); }
        }
        //println!("{} {} {}",BCZ,RXZ,QXZ);
        // update header
        let mut newheader=String::new();
        newheader.push_str(headerpresplit[0]);
        newheader.push_str(" ");
        match pair {
            "R1" => newheader.push_str("1"),
            "R2" => newheader.push_str("2"),
            _ => { /* Unreachable */ },
        }
        newheader.push_str(":N:0:");
        newheader.push_str(&BCZ[5..]);//add from bcz field
        //update sequence and Qualities
        let mut newSequences=String::new();
        let mut newQualities=String::new();
        match pair {
            "R1" => {
                newSequences.push_str(&RXZ[5..8]);
                newQualities.push_str(&QXZ[5..8]);
                },
            "R2" => {
                newSequences.push_str(&RXZ[9..12]);
                newQualities.push_str(&QXZ[9..12]);
                },
            _ => {/* Unreachable */},
        }
        // Push Record Sequence and quality to newSeq and newQual
        newSequences.push_str(str::from_utf8(&c_record.seq).unwrap());
        newQualities.push_str(str::from_utf8(&c_record.qual).unwrap());

        //println!("New Seq {} \nNeq Qual {}",newSequences,newQualities);

        //update the record fields with new values
        c_record.head=newheader.as_bytes().to_vec();
        c_record.seq=newSequences.as_bytes().to_vec();
        c_record.qual=newQualities.as_bytes().to_vec();
        // write to the output buffer
        match c_record.write(&mut out_buf){
            Ok(_) => true,// if writes success fully continue parsing
            _ => false // if write not successfull  stop parsing 
        }
    }
        ).expect("Invalid FASTQ file");
    if readcount==0 {
        // remove the output file
        std::fs::remove_file(outputfilename);
        return(Err(std::io::Error::new(io::ErrorKind::InvalidInput,"0 records parsed in fast file"))) 
    }

    // Flush the remaing of the buffer to the file before exit.
    out_buf.flush().expect("Can't buffer flush to the file");

    println!("Reads {} parsed in file: {:?}",readcount,inputfilename);
    Ok(())
}
fn main() {
    // Parse cli arguments with clap
    let matches= Command::new("Agent2JSI_converter")
        .version("1.0")
        .author("Ibrahim K. <kisakesenhi@gmail.com>")
        .about("Prepends the MBC sequences and qualities to trimmed FASTQ files by Agent Trimmer, new file without Dark bases could be used in JSI software.")
        .arg_required_else_help(true)
        .arg(
            arg!( -f --fastq <FILE> "Fastq files fastq.gz")
            .takes_value(true)
            .multiple(false)
            .hide_short_help(false)
            .value_parser(value_parser!(PathBuf)),
            )
        .arg(
            arg!(-p --pair <MODE> "Read pair for fastq files")
            .takes_value(true)
            .hide_short_help(false)
            .value_parser(["R1","R2"]),
            )
        .get_matches();
    // use with getraw and convert into iter
    if matches.contains_id("fastq"){ // another if
    if matches.value_source("fastq").expect("checked contains_id") == clap::ValueSource::CommandLine {
        let mut fastqfiles_itr = matches.get_raw("fastq")
            .expect("`fastq` is required")
            .into_iter();
        for fq in fastqfiles_itr{
            // Check fastq file and return proper error!
            match check_inputfiles(&PathBuf::from(fq)){
                Ok(outputbuffer) =>{
                    // Get pair
                    match matches.get_one::<String>("pair"){
                        Some(pair) => {
                                match convert_fastq(&PathBuf::from(fq),&outputbuffer,pair) {
                                    Ok(_) =>{},
                                    Err(e) => {eprintln!("Error during conversion of file {:?} with error: {}",fq,e)},
                                    } // match convert_fastq
                                        },
                        _ =>println!("pair info not provided!"),
                    }
                            },
                Err(e)=> eprintln!("Failed to parse file:{:?} with error: {}",fq,e),
                }
            }   
        }
    }
}
