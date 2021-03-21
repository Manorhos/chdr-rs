use chdr::ChdFile;

fn main() {
    let mut args = std::env::args();
    if args.len() < 2 {
        panic!("No CHD filename supplied. Usage: {:?} <CHD file>", std::env::current_exe().unwrap());
    }

    let chd_filename = args.nth(1).unwrap();
    let chd = ChdFile::open(chd_filename).unwrap();

    let tracks = chd.cd_tracks();
    println!("tracks: {:?}", tracks);
}