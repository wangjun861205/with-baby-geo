use crate::core::Indexer;
use anyhow::Error;
use libh3_sys::{degsToRads, edgeLengthKm, geoToH3, kRing, maxKringSize, GeoCoord, H3Index};

pub(crate) struct H3Indexer {
    resolution: i32,
}

impl H3Indexer {
    fn new(resolution: i32) -> Result<Self, Error> {
        if resolution < 0 || resolution > 15 {
            return Err(Error::msg(format!("invalid resolution for h3 indexer: {}", resolution)));
        }
        Ok(Self { resolution })
    }
}

impl Indexer<H3Index> for H3Indexer {
    fn index(&self, latitude: f64, longitude: f64) -> H3Index {
        let coord = GeoCoord {
            lat: unsafe { degsToRads(latitude) },
            lon: unsafe { degsToRads(longitude) },
        };
        unsafe { geoToH3(&coord as *const GeoCoord, self.resolution) }
    }

    fn neighbors(&self, index: H3Index, distance: f64) -> Vec<H3Index> {
        let k = ((distance - unsafe { edgeLengthKm(self.resolution) }) / unsafe { (edgeLengthKm(self.resolution)) * 2.0 }).ceil() as i32;
        let mut res = vec![0u64; unsafe { maxKringSize(k) } as usize];
        unsafe {
            kRing(index, k, &mut res[0] as *mut H3Index);
        }
        res
    }
}
