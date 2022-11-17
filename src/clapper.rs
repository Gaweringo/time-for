use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Inputs {
    /// The query to search a GIF with and add it as a text to that GIF.
    ///
    /// The query string is used to search for a GIF with the TenorAPI.
    /// A random GIF is picked from the first `--considered-gifs` amount of
    /// GIFs. The query is also added as a text to the GIF in the form of:
    /// "time for <QUERY>" if not otherwise specified with `--custom-text`
    pub query: Option<String>,

    /// A custom text to be placed on the query GIF instead of "time for <QUERY>".
    #[arg(short = 't', long = "text")]
    pub custom_text: Option<String>,

    /// The number of gifs in the pool to be considered for the GIF based on the query.
    #[arg(short, long, default_value_t = 5)]
    pub considered_gifs: u8,

    // TODO: Implement --no-upload
    /// Do not upload the GIF to imgur, instead add it directly to the clipboard.
    ///
    /// This will take longer than uploading to imgur, since the file will need to be
    /// converted to the actual GIF format locally instead of having it be done by imgur.
    ///
    /// Pasting the GIF directly can also be a size problem, since e.g. Discord only
    /// accepts 8 MB for non-nitro users which GIFs can easily exceed. Imgur accepts up to
    /// a 200 MB video file, which is what the "GIF" is actually uploaded as.
    #[arg(short, long, default_value_t = false)]
    pub no_upload: bool,

    /// Open the created GIF/webm file in the Windows explorer after creating it.
    #[arg(short = 'x', long, default_value_t = false)]
    pub explorer: bool,

    // TODO: Implement relative folder use
    /// Create the files in relative directory (./time-for) instead of in the
    /// temp directory.
    #[arg(short, long, default_value_t = false)]
    pub relative: bool,

    /// Open the created gif/webm file in the default application.
    #[arg(short, long, default_value_t = false)]
    pub open: bool,
}
