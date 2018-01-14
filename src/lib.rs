use std::env;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub enum MediaKind {
    Movie,
    Music,
    Audiobook,
    MusicVideo,
    TVShow,
    Booklet,
    Rightone,
}

impl MediaKind {
    pub fn as_str(&self) -> &str {
        match *self {
            MediaKind::Movie => "Movie",
            MediaKind::Music => "Music",
            MediaKind::Audiobook => "Audiobook",
            MediaKind::MusicVideo => "Music Video",
            MediaKind::TVShow => "TV Show",
            MediaKind::Booklet => "Booklet",
            MediaKind::Rightone => "Rightone",
        }
    }

    pub fn as_atom(&self) -> Atom {
        Atom::new("Media Kind", self.as_str())
    }
}

#[derive(Debug)]
pub struct Subler {
    pub source: String,
    pub dest: Option<String>,
    pub optimize: bool,
    pub atoms: Atoms,
    pub media_kind: Option<MediaKind>,
}

impl Subler {
    pub fn new(source: &str, atoms: Atoms) -> Self {
        Subler {
            source: source.to_owned(),
            dest: None,
            optimize: false,
            atoms,
            media_kind: None,
        }
    }

    pub fn cli_executeable(&self) -> String {
        env::var("sublercli").unwrap_or(format!("/usr/local/bin/sublercli"))
    }

    ///
    /// Apply the specified metadata to the source file and output it to
    /// the specified destination file
    pub fn tag(&self) -> Result<(), ()> {
        let path = Path::new(self.source.as_str());
        if !path.exists() {
            return Err(());
        }
        let mut atoms = self.atoms.args();

        if self.media_kind.is_some() {
            atoms.push(self.media_kind.as_ref().unwrap().as_atom().arg());
        }

        let meta_tags: Vec<&str> = atoms.iter().map(AsRef::as_ref).collect();

        let mut args = vec!["-source", self.source.as_str()];
        if self.dest.is_some() {
            args.push("-dest");
            args.push(self.dest.as_ref().unwrap().as_str());
        }

        args.extend(meta_tags);

        if self.optimize {
            args.push("-optimize");
        }


        Command::new(self.cli_executeable().as_str())
            .args(&args);


        Ok(())
    }


    pub fn optimize(&mut self, val: bool) -> &mut Self {
        self.optimize = val;
        self
    }

    pub fn media_kind(&mut self, kind: MediaKind) -> &mut Self {
        self.media_kind = Some(kind);
        self
    }

    pub fn dest(&mut self, dest: &str) -> &mut Self {
        self.dest = Some(dest.to_owned());
        self
    }
}

#[derive(Debug, Clone)]
pub struct Atom {
    pub tag: String,
    pub value: String,
}

impl Atom {
    pub fn new(tag: &str, val: &str) -> Atom {
        Atom {
            tag: tag.to_owned(),
            value: val.to_owned(),
        }
    }
    pub fn arg(&self) -> String {
        format!("{{{}:{}}}", self.tag, self.value)
    }
}

macro_rules! atom_tag {

    ( $($ident:tt : $tag:expr),*) => {
        #[derive(Debug, Clone)]
        pub struct Atoms{
            inner: Vec<Atom>,
        }

        impl Atoms {

            /// all valid Metadata Atom tags
            pub fn metadata_params<'a>() -> Vec<&'a str> {
                let mut params = Vec::new();
                $(
                    params.push($tag);
                )*
                params

            }

            $(
                pub fn $ident(&mut self, val: &str){
                    self.inner.push(Atom::new($tag, val));
                }
            )*

            /// args for setting the metaflag flag of subler
            pub fn args(&self) -> Vec<String> {
                let mut args = Vec::new();
                if !self.inner.is_empty(){
                    args.push(format!("-metadata"));
                    args.push(self.inner.iter().map(|x|x.arg()).collect::<Vec<_>>().join(""));
                }
                args
            }

            pub fn add_atom(&mut self, atom: Atom) {
                self.inner.push(atom)
            }

            pub fn add(&mut self, tag: &str, val: &str) {
                self.inner.push(Atom::new(tag, val))
            }
        }

        impl Default for Atoms {
            fn default() -> Atoms {
                Atoms{ inner : Vec::new() }
            }

        }
    };
}

atom_tag!(
    artist: "Artist",
    album_artist: "Album Artist",
    album: "Album",
    grouping: "Grouping",
    composer: "Composer",
    comments: "Comments",
    genre: "Genre",
    release_date: "Release Date",
    track_number: "Track #",
    disk_number: "Disk #",
    tempo: "Tempo",
    tv_show: "TV Show",
    tv_episode_number: "TV Episode #",
    tv_network: "TV Network",
    tv_episode_id: "TV Episode ID",
    tv_season: "TV Season",
    description: "Description",
    long_description: "Long Description",
    series_description: "Series Description",
    hd_video: "HD Video",
    rating_annotation: "Rating Annotation",
    studio: "Studio",
    cast: "Cast",
    director: "Director",
    gapless: "Gapless",
    codirector: "Codirector",
    producers: "Producers",
    screenwriters: "Screenwriters",
    lyrics: "Lyrics",
    copyright: "Copyright",
    encoding_tool: "Encoding Tool",
    encoded_by: "Encoded By",
    keywords: "Keywords",
    category: "Category",
    contentid: "contentID",
    artistid: "artistID",
    playlistid: "playlistID",
    genreid: "genreID",
    composerid: "composerID",
    xid: "XID",
    itunes_account: "iTunes Account",
    itunes_account_type: "iTunes Account Type",
    itunes_country: "iTunes Country",
    track_sub_title: "Track Sub-Title",
    song_description: "Song Description",
    art_director: "Art Director",
    arranger: "Arranger",
    lyricist: "Lyricist",
    acknowledgement: "Acknowledgement",
    conductor: "Conductor",
    linear_notes: "Linear Notes",
    record_company: "Record Company",
    original_artist: "Original Artist",
    phonogram_rights: "Phonogram Rights",
    producer: "Producer",
    performer: "Performer",
    publisher: "Publisher",
    sound_engineer: "Sound Engineer",
    soloist: "Soloist",
    credits: "Credits",
    thanks: "Thanks",
    online_extras: "Online Extras",
    executive_producer: "Executive Producer",
    sort_name: "Sort Name",
    sort_artist: "Sort Artist",
    sort_album_artist: "Sort Album Artist",
    sort_album: "Sort Album",
    sort_composer: "Sort Composer",
    sort_tv_show: "Sort TV Show",
    artwork: "Artwork",
    name: "Name",
    rating: "Rating",
    media_kind: "Media Kind"
   );
