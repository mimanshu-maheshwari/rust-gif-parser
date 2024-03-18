use crate::parser::GifBuffer;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct GifSignature {
    magic: String,
    version: GifVersion,
}

impl GifSignature {
    pub fn parse(buf: &mut GifBuffer) -> Self {
        let magic = String::from_utf8(buf.read_slice(3))
            .map_err(|err| {
                eprintln!("ERROR: Unable to read magic value: {err}");
            })
            .unwrap()
            .to_uppercase();
        assert_eq!("GIF", &magic, "ERROR: Expected 'GIF'  but found '{magic}'");
        let version = String::from_utf8(buf.read_slice(3))
            .map_err(|err| {
                eprintln!("ERROR: Unable to read gif version: {err}");
            })
            .unwrap()
            .into();
        GifSignature { magic, version }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum GifVersion {
    GIF89a,
    GIF87a,
}

impl From<GifVersion> for String {
    fn from(value: GifVersion) -> Self {
        match value {
            GifVersion::GIF89a => String::from("89a"),
            GifVersion::GIF87a => String::from("87a"),
        }
    }
}

impl From<String> for GifVersion {
    fn from(value: String) -> Self {
        let value: &str = &value;
        match value {
            "89a" => GifVersion::GIF89a,
            "87a" => GifVersion::GIF87a,
            _ => panic!("ERROR: GIF version not recognized."),
        }
    }
}

impl From<&str> for GifVersion {
    fn from(value: &str) -> Self {
        match value {
            "89a" => GifVersion::GIF89a,
            "87a" => GifVersion::GIF87a,
            _ => panic!("ERROR: gif version not recognized"),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct LSDPackedFields {
    /// Flag indicating the presence of a Global Color Table; if the flag is set, the Global Color Table will immediately follow the Logical Screen Descriptor.
    /// This flag also selects the interpretation of the Background Color Index; if the flag is set, the value of the Background Color Index field should be used as the table index of the background color.
    /// (This field is the most significant bit of the byte.)
    /// 0 - No Global Color Table follows, the Background Color Index field is meaningless.
    /// 1 - A Global Color Table will immediately follow, the Background Color Index field is meaningful.
    global_color_table_flag: bool,

    /// bits of color resolution
    /// Color Resolution - Number of bits per primary color available to the original image, minus 1.
    /// This value represents the size of the entire palette from which the colors in the graphic were selected, not the number of colors actually used in the graphic.
    /// For example, if the value in this field is 3, then the palette of the original image had 4 bits per primary color available to create the image.
    /// This value should be set to indicate the richness of the original palette, even if not every color from the whole palette is available on the source machine.
    color_resolution: u8,

    /// reserved for future defination and must be 0
    ///Sort Flag - Indicates whether the Global Color Table is sorted.
    ///If the flag is set, the Global Color Table is sorted, in order of decreasing importance.
    ///Typically, the order would be decreasing frequency, with most frequent color first.
    ///This assists a decoder, with fewer available colors, in choosing the best subset of colors; the decoder may use an initial segment of the table to render the graphic.
    /// Values : 0 -   Not ordered.
    ///          1 -   Ordered by decreasing importance, most important color first.
    sort_flag: bool,

    /// bits/pixel in image
    /// Size of Global Color Table - If the Global Color Table Flag is set to 1, the value in this field is used to calculate the number of bytes contained in the Global Color Table.
    /// To determine that actual size of the color table, raise 2 to [the value of the field + 1].
    /// Even if there is no Global Color Table specified, set this field according to the above formula so that decoders can choose the best graphics mode to display the stream in.
    /// (This field is made up of the 3 least significant bits of the byte.)
    global_color_table_size: u8,
}

impl LSDPackedFields {
    pub fn parse(buf: &mut GifBuffer) -> Self {
        let m_u8 = buf.read_u8();
        let global_color_table_flag = (m_u8 >> 7) & 0b1 == 1;
        let color_resolution = ((m_u8 >> 4) & 0b111) + 1u8;
        let sort_flag = (m_u8 >> 3) & 0b1 == 1;
        let global_color_table_size = (m_u8 & 0b111) + 1_u8;
        LSDPackedFields {
            global_color_table_flag,
            color_resolution,
            sort_flag,
            global_color_table_size,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LogicalScreenDescriptor {
    /// Raster width in pixels (LSB first)
    /// Logical Screen Width - Width, in pixels, of the Logical Screen where the images will be rendered in the displaying device.
    logical_screen_width: u16,

    /// Raster height in pixels (LSB first)
    ///Logical Screen Height - Height, in pixels, of the Logical Screen where the images will be rendered in the displaying device.
    logical_screen_height: u16,

    packed_fields: LSDPackedFields,

    /// Color index of screen background
    /// (color is defined from the Global color map or default map if none specified)
    ///Background Color Index - Index into the Global Color Table for the Background Color.
    ///The Background Color is the color used for those pixels on the screen that are not covered by an image.
    ///If the Global Color Table Flag is set to (zero), this field should be zero and should be ignored.
    background_color_index: u8,

    /// Pixel Aspect Ratio - Factor used to compute an approximation of the aspect ratio of the pixel in the original image.  If the value of the field is not 0, this approximation of the aspect ratio is computed based on the formula:
    ///	Aspect Ratio = (Pixel Aspect Ratio + 15) / 64
    /// The Pixel Aspect Ratio is defined to be the quotient of the pixel's width over its height.  The value range in this field allows specification of the widest pixel of 4:1 to the tallest pixel of 1:4 in increments of 1/64th.

    /// Values : 0      -   No aspect ratio information is given.
    ///          1..255 -   Value used in the computation.
    pixel_aspect_ratio: u8,
}

impl LogicalScreenDescriptor {
    pub fn parse(buf: &mut GifBuffer) -> Self {
        let logical_screen_width = buf.read_le_u16();
        let logical_screen_height = buf.read_le_u16();
        let packed_fields = LSDPackedFields::parse(buf);
        let background_color_index = buf.read_u8();
        let pixel_aspect_ratio = buf.read_u8();

        LogicalScreenDescriptor {
            logical_screen_width,
            logical_screen_height,
            packed_fields,
            background_color_index,
            pixel_aspect_ratio,
        }
    }
}

// The Global Color Map is optional but recommended for  images  where
// accurate color rendition is desired.  The existence of this color map is
// indicated in the 'M' field of byte 5 of the Screen Descriptor.

/// This block contains a color table, which is a sequence of bytes representing red-green-blue color triplets.
/// The Global Color Table is used by images without a Local Color Table and by Plain Text Extensions.
/// Its presence is marked by the Global Color Table Flag being set to 1 in the Logical Screen Descriptor; if present, it immediately follows the Logical Screen Descriptor and contains a number of bytes equal to
/// `3 x 2^(Size of Global Color Table+1)`
/// This block is OPTIONAL; at most one Global Color Table may be present per Data Stream.
#[derive(Debug, PartialEq, Eq)]
pub struct GlobalColorMap {
    /// sequentual vector of (r, g, b) values n times
    intensities: Vec<u8>,
    /// size of intensities(r,g,b)
    size: usize,
}

impl GlobalColorMap {
    pub fn parse(buf: &mut GifBuffer, screen_descriptor: &LogicalScreenDescriptor) -> Option<Self> {
        if !screen_descriptor.packed_fields.global_color_table_flag {
            return None;
        }

        let pixel = screen_descriptor.packed_fields.global_color_table_size;
        let size: usize = 3 * 2_usize.pow(pixel as u32);

        let mut intensities: Vec<u8> = vec![0u8; size];

        for i in 0..intensities.len() {
            // intensities[i] = buf.read_u8();
            intensities[i] =
                ((buf.read_u8() as u32 * 255_u32) / ((1_u32 << pixel as u32) - 1_u32)) as u8;
        }

        Some(GlobalColorMap { intensities, size })
    }
}

impl fmt::Display for GlobalColorMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Global Color Map: \n  {{size: {size}}}\n",
            size = self.size
        )?;
        for index in 0..9 {
            let mut prefex = "";
            if index % 3 == 0 {
                write!(f, "\n  {index:#03} => [")?;
            } else {
                prefex = ", ";
            }
            write!(
                f,
                "{prefex}{intensity:#04x}",
                intensity = self.intensities[index]
            )?;
            if index % 3 == 2 {
                write!(f, "]")?;
            }
        }
        for _ in 0..3 {
            write!(f, "\n    ...")?;
        }
        let len = self.intensities.len();
        for index in (0..9).rev() {
            let mut prefex = "";
            let index = len - index - 1;

            if index % 3 == 0 {
                write!(f, "\n  {index:#03} => [")?;
            } else {
                prefex = ", ";
            }

            write!(
                f,
                "{prefex}{intensity:#04x}",
                intensity = self.intensities[index]
            )?;
            if index % 3 == 2 {
                write!(f, "]")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct IDPackedFields {
    //  M=0 - Use global color map, ignore 'pixel'
    //  M=1 - Local color map follows, use 'pixel'
    ///Local Color Table Flag - Indicates the presence of a Local Color Table immediately following this Image Descriptor. (This field is the most significant bit of the byte.)
    ///Values: 0 - Local Color Table is not present. Use Global Color Table if available.
    ///        1 - Local Color Table present, and to follow immediately after this Image Descriptor.
    local_color_table_flag: bool,

    // I=0 - Image formatted in Sequential order
    // I=1 - Image formatted in Interlaced order
    /// Interlace Flag - Indicates if the image is interlaced. An image is interlaced in a four-pass interlace pattern; see Appendix E for details.
    /// Values: 0 - Image is not interlaced.
    ///         1 - Image is interlaced.
    interlace_flag: bool,

    ///Sort Flag - Indicates whether the Local Color Table is sorted.  If the flag is set, the Local Color Table is sorted, in order of decreasing importance. Typically, the order would be decreasing frequency, with most frequent color first. This assists a decoder, with fewer available colors, in choosing the best subset of colors; the decoder may use an initial segment of the table to render the graphic.
    /// Values: 0 -   Not ordered.
    ///         1 -   Ordered by decreasing importance, most important color first.
    sort_flag: bool,

    /// Size of Local Color Table - If the Local Color Table Flag is set to 1, the value in this field is used to calculate the number of bytes contained in the Local Color Table. To determine that actual size of the color table, raise 2 to the value of the field + 1. This value should be 0 if there is no Local Color Table specified. (This field is made up of the 3 least significant bits of the byte.)
    reserved: u8,

    // pixel+1 - # bits per pixel for this image
    /// Extensions and Scope. The scope of this block is the Table-based Image Data Block that follows it. This block may be modified by the Graphic Control Extension.
    local_color_table_size: u8,
}

// 0 1 2 3 4 5 6 7
// 7 6 5 4 3 2 1 0

impl IDPackedFields {
    pub fn parse(buf: &mut GifBuffer) -> Self {
        let m_u8: u8 = buf.read_u8();
        let local_color_table_flag = (m_u8 >> 7) & 0b1 == 1;
        let interlace_flag = (m_u8 >> 6) & 0b1 == 1;
        let sort_flag = (m_u8 >> 5) & 0b1 == 1;
        let reserved = (m_u8 >> 4) & 0b11;
        let local_color_table_size: u8 = (m_u8 & 0b111) + 1_u8;

        IDPackedFields {
            local_color_table_flag,
            interlace_flag,
            sort_flag,
            reserved,
            local_color_table_size,
        }
    }
}

/// Each image in the Data Stream is composed of an Image Descriptor, an optional Local Color Table, and the image data.
/// Each image must fit within the boundaries of the Logical Screen, as defined in the Logical Screen Descriptor.
/// The Image Descriptor contains the parameters necessary to process a table based image.
/// The coordinates given in this block refer to coordinates within the Logical Screen, and are given in pixels.
/// This block is a Graphic-Rendering Block, optionally preceded by one or more Control blocks such as the Graphic Control Extension, and may be optionally followed by a Local Color Table; the Image Descriptor is always followed by the image data.
/// This block is REQUIRED for an image.
/// Exactly one Image Descriptor must be present per image in the Data Stream.
/// An unlimited number of images may be present per Data Stream.
#[derive(Debug, PartialEq, Eq)]
pub struct ImageDescriptor {
    /// Identifies the beginning of an Image Descriptor. This field contains the fixed value 0x2C.
    // Start of image in pixels from the left side of the screen (LSB first);
    /// Image Left Position - Column number, in pixels, of the left edge of the image, with respect to the left edge of the Logical Screen. Leftmost column of the Logical Screen is 0.
    image_left: u16,

    // Start of image in pixels from the top of the screen (LSB first)
    ///  Image Top Position - Row number, in pixels, of the top edge of the image with respect to the top edge of the Logical Screen. Top row of the Logical Screen is 0.
    image_top: u16,

    // Width of the image in pixels (LSB first)
    /// Image Width - Width of the image in pixels.
    image_width: u16,

    // Height of the image in pixels (LSB first)
    /// Image Height - Height of the image in pixels.
    image_height: u16,

    packed_fields: IDPackedFields,
}
impl ImageDescriptor {
    pub fn parse(buf: &mut GifBuffer) -> Self {
        let image_separator: u8 = buf.read_u8();
        assert_eq!(
            0x2C, image_separator,
            "ERROR: Expected \",\" or \"0x2C\" but found {image_separator:#04x}"
        );
        let image_left = buf.read_le_u16();
        let image_top = buf.read_le_u16();
        let image_width = buf.read_le_u16();
        let image_height = buf.read_le_u16();
        let packed_fields = IDPackedFields::parse(buf);
        ImageDescriptor {
            image_left,
            image_top,
            image_width,
            image_height,
            packed_fields,
        }
    }
}

/// This block contains a color table, which is a sequence of bytes representing red-green-blue color triplets. The Local Color Table is used by the image that immediately follows. Its presence is marked by the Local Color Table Flag being set to 1 in the Image Descriptor; if present, the Local Color Table immediately follows the Image Descriptor and contains a number of bytes equal to
///    `3x2^(Size of Local Color Table+1)`
///If present, this color table temporarily becomes the active color table and the following image should be processed using it. This block is OPTIONAL; at most one Local Color Table may be present per Image Descriptor and its scope is the single image associated with the Image Descriptor that precedes it.
#[derive(Debug, PartialEq, Eq)]
pub struct LocalColorMap {}
impl LocalColorMap {
    pub fn parse(_buf: &mut GifBuffer, image_descriptor: &ImageDescriptor) -> Option<Self> {
        if !image_descriptor.packed_fields.local_color_table_flag {
            return None;
        }

        Some(LocalColorMap {})
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct RasterData {}
impl RasterData {
    pub fn parse(_buf: &mut GifBuffer, image_descriptor: &ImageDescriptor) -> Self {
        if image_descriptor.packed_fields.interlace_flag {}
        RasterData {}
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct DescriptorGroup {
    pub image_descriptor: ImageDescriptor,
    pub local_color_map: Option<LocalColorMap>,
    pub raster_data: RasterData,
}

impl DescriptorGroup {
    fn parse(buf: &mut GifBuffer) -> Self {
        let image_descriptor: ImageDescriptor = ImageDescriptor::parse(buf);
        let local_color_map: Option<LocalColorMap> = LocalColorMap::parse(buf, &image_descriptor);
        let raster_data: RasterData = RasterData::parse(buf, &image_descriptor);

        DescriptorGroup {
            image_descriptor,
            local_color_map,
            raster_data,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Terminator {}
impl Terminator {
    pub fn parse(_buf: &mut GifBuffer) -> Self {
        Terminator {}
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Gif {
    pub signature: GifSignature,
    pub logical_screen_descriptor: LogicalScreenDescriptor,
    pub global_color_map: Option<GlobalColorMap>,
    pub descriptor_groups: Vec<DescriptorGroup>,
    // pub terminator: Terminator,
}

impl Gif {
    pub fn decode(file_path: &str) -> Self {
        let mut buf = GifBuffer::read(file_path);
        let signature = GifSignature::parse(&mut buf);
        println!("INFO: Magic value: {signature:#?}");

        // assert_eq!( GifVersion::GIF87a, signature.version, "ERROR: Program only works with GIF87a version");

        let logical_screen_descriptor = LogicalScreenDescriptor::parse(&mut buf);
        println!("INFO: Screen Descriptor: {logical_screen_descriptor:#?}");

        let global_color_map = GlobalColorMap::parse(&mut buf, &logical_screen_descriptor);
        if let Some(gcm) = &global_color_map {
            println!("INFO: {gcm}");
        }

        let mut descriptor_groups: Vec<DescriptorGroup> = Vec::new();
        // while the terminator bit (0x3B) or ';' is not found
        // read the descriptors
        // while buf.peek_u8() !=  0x3B {
        let descriptor_group: DescriptorGroup = DescriptorGroup::parse(&mut buf);
        descriptor_groups.push(descriptor_group);
        // }
        println!("INFO: Image Descriptors: {descriptor_groups:#?}");
        Gif {
            signature,
            logical_screen_descriptor,
            global_color_map,
            descriptor_groups,
        }
    }
}
