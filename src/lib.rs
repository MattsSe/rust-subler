//! sublercli-rs - a simple commandline interface for the sublerCLI tool on mac OS
//! to write metadata to media files
//!
//! ## Installation
//!
//! Requires an additional [SublerCLI](https://bitbucket.org/galad87/sublercli) Installation.
//! To install with homebrew: `brew cask install sublercli`
//!
//! By default `sublercli-rs` assumes a `homebrew` installation under /usr/local/bin/SublerCli`
//! You can check your installtion path with `brew cask info sublercli`
//! If the SublerCLI installation destination deviates from default, you can overwerite the path
//! by setting the `SUBLER_CLI_PATH` environment variable to the valid destination.
//!
//! ## Atoms
//!
//! To store metadata, Atoms are used. An Atom has a specifc name and the value it stores.
//! The `Atom` struct mimics this behavior. There is a predefined set of valid atoms.
//! To obtain a list of al valid metadata atom tag names:
//!
//! ```rust,no_run
//! use sublercli::Atoms;
//! let valid_tags: Vec<&str> = Atoms::metadata_tags();
//! ```
//! Support for the predefined set of known atoms is individually implemented.
//! `Atoms` functions as a wrapper to store a set of single `Atom` values and is used
//! to create Atoms like:
//!
//! ```rust,no_run
//! use sublercli::*;
//! let atoms = Atoms::new()
//!     .add("Cast", "John Doe")
//!     .genre("Foo,Bar")
//!     .artist("Foo Artist")
//!     .title("Foo Bar Title")
//!     .release_date("2018")
//!     .build();
//! ```
//!
//! ## Tagging
//!
//! To invoke the SublerCLI process:
//! If no dest path is supplied then the destination path is the existing file name
//! suffixed, starting from 0: `demo.mp4 -> demo.0.mp4`
//! ```rust,no_run
//! use sublercli::*;   
//! let file = "demo.mp4";
//! let subler = Subler::new(file, Atoms::new().title("Foo Bar Title").build())
//!     // by default, mediakind is already set to `Movie`
//!     .media_kind(Some(MediaKind::Movie))
//!
//!     // set an optional destination path
//!     .dest("dest/path")
//!
//!     // by default the optimization flag is set to true
//!     .optimize(false)
//!
//!     // execute prcess in sync,
//!     // alternativly spawn the process: `.spawn_tag()`
//!     .tag()
//!
//!     .and_then(|x| {
//!         println!("stdout: {}", String::from_utf8_lossy(&x.stdout));
//!         Ok(())
//!     });
//! ```

#![deny(warnings)]
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output};

/// Represents the type of media for a input file
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
        match self {
            MediaKind::Movie => "Movie",
            MediaKind::Music => "Music",
            MediaKind::Audiobook => "Audiobook",
            MediaKind::MusicVideo => "Music Video",
            MediaKind::TVShow => "TV Show",
            MediaKind::Booklet => "Booklet",
            MediaKind::Rightone => "Rightone",
        }
    }

    /// Creates a new `Media Kind`
    pub fn as_atom(&self) -> Atom {
        Atom::new("Media Kind", self.as_str())
    }
}

#[derive(Debug)]
pub struct Subler {
    /// The path to the source file
    pub source: String,
    /// The path to the destination file
    pub dest: Option<String>,
    /// The Subler optimization flag
    pub optimize: bool,
    /// The atoms that should be written to the file
    pub atoms: Atoms,
    /// The Mediakind of the file
    pub media_kind: Option<MediaKind>,
}

impl Subler {
    /// creates a new SublerCli Interface with a set of Atoms that
    /// should be set to the the file at the `source`
    /// By default MediaKind is set to `MediaKind::Movie` and
    /// optimization level is set to true
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
    /// Assumes a homebrew installtion by default under `/usr/local/bin/SublerCli`,
    /// can be overwritten setting the `SUBLER_CLI_PATH` env variable
    pub fn cli_executeable() -> String {
        ::std::env::var("SUBLER_CLI_PATH").unwrap_or_else(|_| "/usr/local/bin/SublerCli".to_owned())
    }

    /// Executes the tagging command as a child process, returning a handle to it.
    pub fn spawn_tag(&mut self) -> io::Result<Child> {
        let mut cmd = self.build_tag_command()?;
        cmd.spawn()
    }

    /// create the subler process command
    pub fn build_tag_command(&mut self) -> io::Result<Command> {
        let path = Path::new(self.source.as_str());
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Source file does not exist.".to_owned(),
            ));
        }
        if let Some(ref media_kind) = self.media_kind {
            self.atoms.add_atom(media_kind.as_atom());
        }

        let dest = self
            .determine_dest()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Dest Not Found!"))?;
        let atoms = self.atoms.args();
        let mut args = vec!["-source", self.source.as_str()];
        args.push("-dest");
        args.push(dest.as_str());
        let meta_tags: Vec<&str> = atoms.iter().map(AsRef::as_ref).collect();
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

    /// sets the optimization flag
    pub fn optimize(&mut self, val: bool) -> &mut Self {
        self.optimize = val;
        self
    }

    pub fn media_kind(&mut self, kind: Option<MediaKind>) -> &mut Self {
        self.media_kind = kind;
        self
    }

    /// sets the destination of the output file
    pub fn dest(&mut self, dest: &str) -> &mut Self {
        self.dest = Some(dest.to_owned());
        self
    }

    /// computes the next available path by appending
    fn next_available_path(&self, p: &str, i: i32) -> Option<PathBuf> {
        let path = Path::new(p);
        let parent = path.parent()?.to_str()?;
        let stem = path.file_stem()?.to_str()?;
        let extension = path.extension()?.to_str()?;
        let dest = format!("{}/{}.{}.{}", parent, stem, i, extension);
        let new_path = Path::new(dest.as_str());
        if new_path.exists() {
            self.next_available_path(p, i + 1)
        } else {
            Some(new_path.to_owned())
        }
    }

    /// finds the next valid destination path, if no dest path is supplied
    /// then the destination path is the existing file name suffixed, starting from 0
    fn determine_dest(&self) -> Option<String> {
        match self.dest {
            Some(ref s) => {
                let p = Path::new(s.as_str());
                if p.exists() {
                    Some(
                        self.next_available_path(s.as_str(), 0)?
                            .to_str()?
                            .to_owned(),
                    )
                } else {
                    Some(s.clone())
                }
            }
            _ => Some(
                self.next_available_path(self.source.as_str(), 0)?
                    .to_str()?
                    .to_owned(),
            ),
        }
    }
}

/// Represents a Metadata Media Atom
#[derive(Debug, Clone)]
pub struct Atom {
    /// The Name of the Metadata Atom
    pub tag: String,
    /// The Value this atom contains
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
            /// The stored atoms
            inner: Vec<Atom>,
        }

        impl Atoms {

            pub fn new() -> Builder {
                Builder::default()
            }

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
                    args.push("-metadata".to_owned());
                    args.push(self.inner.iter().map(Atom::arg).collect::<Vec<_>>().join(""));
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

            pub fn atoms_mut(&mut self) -> &mut Vec<Atom> {
                &mut self.inner
            }
        }

        #[derive(Debug)]
        pub struct Builder {
            pub atoms: Vec<Atom>,
        }

        impl Builder {

             $(
                pub fn $ident(&mut self, val: &str) -> &mut Self{
                    self.atoms.push(Atom::new($tag, val));
                    self
                }
            )*

            pub fn add_atom(&mut self, atom: Atom) -> &mut Self {
                self.atoms.push(atom);
                self
            }

            pub fn add(&mut self, tag: &str, val: &str) -> &mut Self {
                self.atoms.push(Atom::new(tag, val));
                self
            }

            pub fn build(&self) -> Atoms {
                Atoms {inner: self.atoms.clone()}
            }
        }

        impl Default for Builder {
            fn default() -> Self {
                Builder { atoms: Vec::new() }
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
