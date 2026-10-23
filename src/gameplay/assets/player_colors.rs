//! Read-only original player-slot palette; no copied table or inferred nation colors.
use super::{asset_error,pixel::{expand5,expand6},text,tile::PixelFormat};
use crate::gameplay::GameplayError;
use std::{fs::File,io::Read,path::Path};

/// Installed default color set; the client does not expose alternate sets yet.
const FILE:&str="ctp2_data/default/gamedata/Colors00.txt";
/// Nine general colors precede player0; enum names are one-based, file comments zero-based.
const PLAYER_OFFSET:usize=9_usize;
/// Released colorset.h supports17 player colors including barbarian slot0.
const PLAYER_COUNT:usize=17_usize;
/// Hard bound for this tiny visual metadata file.
const MAX_BYTES:u64=0x0001_0000_u64;
/// Owned RGB rows read once from the selected installation, not generated content.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct PlayerPalette {
    /// Only source player0..16 RGB entries, in their original slot order.
    colors:Vec<[u8;3]>,
}
impl PlayerPalette {
    /* Released ctp2_source/ctp2_code/gfx/gfx_utils/ColorSet.cpp::{Import,
     * ComputePlayerColor,GetPlayerColor,GetDarkPlayerColor}, colorset.h::COLOR:
     * COLOR_PLAYER1+playerNum indexes Colors00's player0..16 after9 generic rows.
     * ctp2/ctp2_data/default/gamedata/Colors00.txt is read, never embedded/copied.
     * Import packs565 then pixelutils_Convert565to555 conditionally; remembered
     * city colors use pixelutils_Shadow's per-channel half, not an RGB tint.
     * SDL RGBA expands the same packed channels via existing original decoders.
     */
    /// Parse the complete bounded palette; never accept a shifted/partial color table.
    /// # Errors
    /// Rejects malformed count/grammar/channels or missing player rows.
    pub fn parse(input:&str)->Result<Self,GameplayError> {
        if u64::try_from(input.len()).unwrap_or(u64::MAX)>MAX_BYTES{return Err(asset_error("player palette exceeds64KiB".to_owned()));}
        let mut tokens=input.lines().flat_map(|line|line.split('#').next().unwrap_or_default().split_whitespace());
        let count=tokens.next().and_then(|value|value.parse::<usize>().ok()).filter(|count|(PLAYER_OFFSET.saturating_add(PLAYER_COUNT)..100_usize).contains(count)).ok_or_else(||asset_error("invalid original player palette count".to_owned()))?;
        if tokens.next()!=Some("COLORSET")||tokens.next()!=Some("{"){return Err(asset_error("invalid original player palette header".to_owned()));}
        let mut colors=Vec::with_capacity(PLAYER_COUNT);
        for index in 0_usize..count {
            if tokens.next()!=Some("COLORSET_COLOR"){return Err(asset_error("truncated original player palette".to_owned()));}
            let mut rgb=[0_u8;3_usize];
            for channel in &mut rgb {*channel=tokens.next().and_then(|value|value.parse::<u8>().ok()).ok_or_else(||asset_error("invalid original player palette channel".to_owned()))?;}
            if (PLAYER_OFFSET..PLAYER_OFFSET.saturating_add(PLAYER_COUNT)).contains(&index){colors.push(rgb);}
        }
        if tokens.next()!=Some("}")||tokens.next().is_some(){return Err(asset_error("invalid original player palette terminator".to_owned()));}
        Ok(Self{colors})
    }
    /// Load once from the actual installation, including original ANSI comments.
    /// # Errors
    /// Reports missing/unreadable/malformed palette instead of fabricating colors.
    pub fn load(root:&Path)->Result<Self,GameplayError> {
        let mut bytes=Vec::new();File::open(root.join(FILE)).map_err(|error|asset_error(format!("original player palette: {error}")))?.take(MAX_BYTES.saturating_add(1_u64)).read_to_end(&mut bytes).map_err(|error|asset_error(format!("original player palette: {error}")))?;
        Self::parse(&text::decode(&bytes))
    }
    /// Exact original packed-channel normal/remembered color; overflow stays unknown.
    #[must_use]
    pub fn color(&self,slot:u8,format:PixelFormat,explored:bool)->Option<[u8;3]> {
        let rgb=self.colors.get(usize::from(slot))?;
        let green_shift=if format==PixelFormat::Rgb565 {2_u8}else{3_u8};
        let mut packed=[rgb[0_usize]>>3_u8,rgb[1_usize]>>green_shift,rgb[2_usize]>>3_u8];
        if explored {for channel in &mut packed {*channel=channel.div_euclid(2_u8);}}
        Some([expand5(packed[0_usize]),if format==PixelFormat::Rgb565 {expand6(packed[1_usize])}else{expand5(packed[1_usize])},expand5(packed[2_usize])])
    }
}

#[cfg(test)]
mod tests {
    use super::PlayerPalette;
    use crate::gameplay::assets::tile::PixelFormat;
    use alloc::collections::BTreeSet;
    use core::fmt::Write as _;
    use std::path::Path;
    /// Artificial rows, not a copied original color table.
    fn synthetic()->String {
        let mut rows=String::from("26\nCOLORSET\n{\n");
        for index in 0_u8..26_u8 {writeln!(&mut rows,"COLORSET_COLOR {} {} {}",index.wrapping_mul(9_u8),index.wrapping_mul(7_u8),index.wrapping_mul(5_u8)).expect("in-memory formatting");}
        rows.push('}');rows
    }
    #[test]
    fn player_colors_original_palette_reads_all_distinct_player_slots_without_copying_assets() {
        let palette=PlayerPalette::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("ctp2")).expect("read-only original Colors00");
        let colors:BTreeSet<_>=(0_u8..17_u8).map(|slot|palette.color(slot,PixelFormat::Rgb565,false).expect("source slot")).collect();assert_eq!(colors.len(),17_usize,"barbarians and16 civilization display slots are distinct");assert_eq!(palette.color(17_u8,PixelFormat::Rgb565,false),None,"never wrap or reuse a color");
    }
    #[test]
    fn player_colors_palette_boundaries_and_packed_fog_match_original_channel_rules() {
        let input=synthetic();let palette=PlayerPalette::parse(&input).expect("synthetic grammar");
        for format in [PixelFormat::Rgb565,PixelFormat::Rgb555] {
            let normal=palette.color(1_u8,format,false).expect("slot");let fog=palette.color(1_u8,format,true).expect("fog slot");assert!(fog.iter().zip(normal).all(|(dark,light)|*dark<light),"each packed channel halves before expansion");
        }
        for invalid in [input.replace("COLORSET_COLOR 0 0 0","COLORSET_COLOR 256 0 0"),input.replace("26\n","25\n"),input.trim_end_matches('}').to_owned(),format!("{input} trailing"),"x".repeat(0x0001_0001_usize)] {assert!(PlayerPalette::parse(&invalid).is_err(),"malformed palette cannot shift/reuse another player's color");}
    }
}
