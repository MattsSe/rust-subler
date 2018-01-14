# rust-subler

Simple rust interface to the SublerCLI tool on mac OS to edit metadata
tagging with [SublerCLI](https://bitbucket.org/galad87/sublercli).

### Installation
Due to SublerCli `rust-subler` is only supported on mac OS.
You need SublerCli installed.

Install with brew:
```
brew cask install sublercli
``` 
`rust-subler` will assume sublercli is installed at `/usr/local/bin/SublerCli`, check with `brew cask info sublercli`.

If installed elsewhere set the `SUBLER_CLI_PATH` environment variable for it.

Add to cargo project:

```
[dependencies.rust-subler]
git = "https://github.com/MattsSe/rust-subler"
``` 


### Atoms
Setting metadata tags is done via Atoms. The `Atoms` struct has a dedicated function for each valid tag.

```
let mut atoms = Atoms::default();
atoms.artist("FooArtist")
     .album("BarAlbum")
     .genre("Baz")
     /*...*/;
```
You can also get a list of the available tags with:
```
let valid_tags: Vec<&str> = Atoms::metadata_tags();
```

### Tagging
Invoke the SublerCLI:
```
use rust_subler::*;

let source_file = "path/to/file"

Subler::new(source_file, atoms)
        // .media_kind(Some(MediaKind::Music)) by default mediakind is set to MediaKind::Movie
        // .optimize(false) // default optimization is set to true
        .dest("dest/path")
        .tag();
        // alternativly spawn the process: .spawn_tag()

```


