use text_io::try_scan;

pub const CDROM_TRACK_METADATA_TAG: u32 = (b'C' as u32) << 24
    | (b'H' as u32) << 16
    | (b'T' as u32) << 8
    | (b'R' as u32);

pub const CDROM_TRACK_METADATA2_TAG: u32 = (b'C' as u32) << 24
    | (b'H' as u32) << 16
    | (b'T' as u32) << 8
    | (b'2' as u32);

#[derive(Debug)]
pub struct CdTrackInfo {
    pub track_no: u8,
    pub track_type: String,
    pub sub_type: String,
    pub frames: u32,

    // These are only present when using the "new" metadata format
    pub pregap: Option<u32>,
    pub pgtype: Option<String>,
    pub pgsub: Option<String>,
    pub postgap: Option<u32>,
}

impl CdTrackInfo {
    pub(super) fn from_old_metadata(bytes: &[u8]) -> Result<CdTrackInfo, text_io::Error> {
        let track_no;
        let track_type;
        let sub_type;
        let frames;

        try_scan!(bytes.iter().copied() => "TRACK:{} TYPE:{} SUBTYPE:{} FRAMES:{}",
            track_no, track_type, sub_type, frames
        );

        Ok(CdTrackInfo {
            track_no,
            track_type,
            sub_type,
            frames,
            pregap: None,
            pgtype: None,
            pgsub: None,
            postgap: None,
        })
    }

    pub(super) fn from_metadata(bytes: &[u8]) -> Result<CdTrackInfo, text_io::Error> {
        let track_no;
        let track_type;
        let sub_type;
        let frames;
        let pregap;
        let pgtype;
        let pgsub;
        let postgap;

        try_scan!(bytes.iter().copied() => "TRACK:{} TYPE:{} SUBTYPE:{} FRAMES:{} \
            PREGAP:{} PGTYPE:{} PGSUB:{} POSTGAP:{}",
            track_no, track_type, sub_type, frames,
            pregap, pgtype, pgsub, postgap
        );

        Ok(CdTrackInfo {
            track_no,
            track_type,
            sub_type,
            frames,
            pregap: Some(pregap),
            pgtype: Some(pgtype),
            pgsub: Some(pgsub),
            postgap: Some(postgap),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_old_metadata()  {
        let s = b"TRACK:1 TYPE:MODE2_RAW SUBTYPE:NONE FRAMES:68352";

        let info = CdTrackInfo::from_old_metadata(s).unwrap();

        assert_eq!(info.track_no, 1);
        assert_eq!(info.track_type, "MODE2_RAW");
        assert_eq!(info.sub_type, "NONE");
        assert_eq!(info.frames, 68352);
        assert_eq!(info.pregap, None);
        assert_eq!(info.pgtype, None);
        assert_eq!(info.pgsub, None);
        assert_eq!(info.postgap, None);
    }

    #[test]
    fn test_from_metadata()  {
        let s = b"TRACK:1 TYPE:MODE2_RAW SUBTYPE:NONE FRAMES:68352 \
            PREGAP:0 PGTYPE:MODE1 PGSUB:NONE POSTGAP:0";

        let info = CdTrackInfo::from_metadata(s).unwrap();

        assert_eq!(info.track_no, 1);
        assert_eq!(info.track_type, "MODE2_RAW");
        assert_eq!(info.sub_type, "NONE");
        assert_eq!(info.frames, 68352);
        assert_eq!(info.pregap.unwrap(), 0);
        assert_eq!(info.pgtype.unwrap(), "MODE1");
        assert_eq!(info.pgsub.unwrap(), "NONE");
        assert_eq!(info.postgap.unwrap(), 0);
    }
}
