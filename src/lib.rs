use std::env;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output};
use std::io;

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
            optimize: true,
            atoms,
            media_kind: Some(MediaKind::Movie),
        }
    }

    /// returns the path to the sublercli executeable
    pub fn cli_executeable() -> String {
        env::var("SUBLER_CLI_PATH").unwrap_or(format!("/usr/local/bin/SublerCli"))
    }

    /// Executes the tagging command as a child process, returning a handle to it.
    pub fn spawn_tag(&mut self) -> io::Result<Child> {
        let mut cmd = self.build_tag_command()?;
        cmd.spawn()
    }

    /// create the subler process command
    fn build_tag_command(&mut self) -> io::Result<Command> {
        let path = Path::new(self.source.as_str());
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Source file does not exist."),
            ));
        }

        let dest = self.determine_dest();

        if self.media_kind.is_some() {
            self.atoms
                .add_atom(self.media_kind.as_ref().unwrap().as_atom());
        }
        let mut atoms = self.atoms.args();

        let meta_tags: Vec<&str> = atoms.iter().map(AsRef::as_ref).collect();
        let mut args = vec!["-source", self.source.as_str()];
        args.push("-dest");
        args.push(dest.as_str());

        args.extend(meta_tags);

        if self.optimize {
            args.push("-optimize");
        }

        let mut cmd = Command::new(Subler::cli_executeable().as_str());
        cmd.args(&args);
        Ok(cmd)
    }

    /// Apply the specified metadata to the source file and output it to
    /// the specified destination file
    pub fn tag(&mut self) -> io::Result<Output> {
        let mut cmd = self.build_tag_command()?;
        cmd.output()
    }

    pub fn optimize(&mut self, val: bool) -> &mut Self {
        self.optimize = val;
        self
    }

    pub fn media_kind(&mut self, kind: Option<MediaKind>) -> &mut Self {
        self.media_kind = kind;
        self
    }

    pub fn dest(&mut self, dest: &str) -> &mut Self {
        self.dest = Some(dest.to_owned());
        self
    }

    /// find the next available path
    fn next_path(&self, p: &str, i: i32) -> PathBuf {
        let path = Path::new(p);
        let parent = path.parent().unwrap().to_str().unwrap();
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let extension = path.extension().unwrap().to_str().unwrap();
        let dest = format!("{}/{}.{}.{}", parent, stem, i, extension);
        let new_path = Path::new(dest.as_str());
        if new_path.exists() {
            self.next_path(p, i + 1)
        } else {
            new_path.to_owned()
        }
    }

    pub fn determine_dest(&self) -> String {
        match self.dest {
            Some(ref s) => {
                let p = Path::new(s.as_str());
                if p.exists() {
                    self.next_path(s.as_str(), 0).to_str().unwrap().to_owned()
                } else {
                    s.clone()
                }
            }
            _ => self.next_path(self.source.as_str(), 0).to_str().unwrap().to_owned()
        }
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
        pub struct Atoms {
            inner: Vec<Atom>,
        }

        impl Atoms {

            /// all valid Metadata Atom tags
            pub fn metadata_tags<'a>() -> Vec<&'a str> {
                let mut params = Vec::new();
                $(
                    params.push($tag);
                )*
                params

            }

            $(
                pub fn $ident(&mut self, val: &str) -> &mut Self{
                    self.inner.push(Atom::new($tag, val));
                    self
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

            pub fn add_atom(&mut self, atom: Atom) -> &mut Self {
                self.inner.push(atom);
                self
            }

            pub fn add(&mut self, tag: &str, val: &str) -> &mut Self {
                self.inner.push(Atom::new(tag, val));
                self
            }

            pub fn atoms(&self) -> &Vec<Atom> {
                &self.inner
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
    title: "Name",
    rating: "Rating",
    media_kind: "Media Kind"
   );
