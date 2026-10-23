//! Revision-bound lookup avoids scanning every explored tile for each drawn edge.
use crate::gameplay::{
    model::MapPoint,
    server::{ServerProjection, ServerTileProjection},
};
use alloc::collections::{BTreeMap,BTreeSet};

/// Coordinate/visible-city ID indices only; no duplicate geography or hidden facts.
#[derive(Debug, Default)]
pub(super) struct ObservedTiles {
    /// Revision of this session's immutable viewer snapshot.
    revision: Option<i64>,
    /// Explicit observed coordinates mapped back into that snapshot.
    indices: BTreeMap<(u16, u16), usize>,
    /// Exact currently visible city IDs only; influence does not reveal hidden city facts.
    influence_cities:BTreeSet<String>,
}
impl ObservedTiles {
    /// Rebuilds once per complete snapshot, never once per tile or pointer lookup.
    pub(super) fn update(&mut self, viewer: &ServerProjection) {
        if self.revision != Some(viewer.revision) {
            self.indices = viewer
                .tiles
                .iter()
                .enumerate()
                .map(|(index, tile)| ((tile.x, tile.y), index))
                .collect();
            let visible_tiles:BTreeSet<_>=viewer.tiles.iter().filter(|tile|!tile.stale&&!tile.id.is_empty()).map(|tile|tile.id.as_str()).collect();
            self.influence_cities=viewer.cities.iter().filter(|city|!city.stale&&!city.id.is_empty()&&!city.tile_id.is_empty()&&visible_tiles.contains(city.tile_id.as_str())).map(|city|city.id.clone()).collect();
            self.revision = Some(viewer.revision);
        }
    }
    /// Returns the actual observed record, never a neighbor or manufactured cell.
    pub(super) fn get<'viewer>(
        &self,
        viewer: &'viewer ServerProjection,
        point: MapPoint,
    ) -> Option<&'viewer ServerTileProjection> {
        self.indices
            .get(&(point.x, point.y))
            .and_then(|index| viewer.tiles.get(*index))
            .filter(|tile| tile.x == point.x && tile.y == point.y)
    }
}

impl super::GameplayTextures<'_> {
    /// Last-observed influence IDs do not authorize hidden/current city lookups.
    pub(in crate::gameplay::runtime) fn influence_city_visible(&self,id:&str)->bool {self.observed.influence_cities.contains(id)}
    /// Updates only coordinate indices, never copies or extrapolates tile facts.
    pub(in crate::gameplay::runtime) fn observe_tiles(&mut self, viewer: &ServerProjection) {
        self.observed.update(viewer);
    }
    /// Returns the snapshot's own cached record at exactly the observed point.
    pub(in crate::gameplay::runtime) fn observed_tile<'viewer>(
        &self,
        viewer: &'viewer ServerProjection,
        point: MapPoint,
    ) -> Option<&'viewer ServerTileProjection> {
        self.observed.get(viewer, point)
    }
}

#[cfg(test)]
mod tests {
    use super::ObservedTiles;
    use crate::gameplay::{
        model::MapPoint,
        server::{ServerProjection, ServerTileProjection},
    };
    #[test]
    fn index_contains_only_observations_and_replaces_indices_with_new_revisions() {
        let mut viewer = ServerProjection {
            revision: 1,
            tiles: vec![ServerTileProjection {
                x: 1,
                y: 1,
                ..ServerTileProjection::default()
            }],
            ..ServerProjection::default()
        };
        let mut index = ObservedTiles::default();
        index.update(&viewer);
        assert!(
            index.get(&viewer, MapPoint::new(1, 1)).is_some(),
            "actual observed cell"
        );
        assert!(
            index.get(&viewer, MapPoint::new(2, 1)).is_none(),
            "hidden cell cannot be manufactured"
        );
        viewer.revision = 2;
        viewer.tiles[0].x = 2;
        index.update(&viewer);
        assert!(
            index.get(&viewer, MapPoint::new(1, 1)).is_none(),
            "old location removed completely"
        );
        assert!(
            index.get(&viewer, MapPoint::new(2, 1)).is_some(),
            "new snapshot record returned"
        );
    }
}
