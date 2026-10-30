//! Pure layout policy: each group collapses before the ribbon can overflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Density {
    Expanded,
    Compact,
    Collapsed,
}
#[derive(Debug, Clone, Copy)]
pub struct Group {
    pub expanded: f32,
    pub compact: f32,
    pub collapsed: f32,
    pub priority: u8,
}
pub const GROUPS: [Group; 6] = [
    Group {
        expanded: 132.,
        compact: 108.,
        collapsed: 74.,
        priority: 3,
    },
    Group {
        expanded: 242.,
        compact: 194.,
        collapsed: 82.,
        priority: 0,
    },
    Group {
        expanded: 220.,
        compact: 168.,
        collapsed: 90.,
        priority: 2,
    },
    Group {
        expanded: 216.,
        compact: 142.,
        collapsed: 76.,
        priority: 1,
    },
    Group {
        expanded: 250.,
        compact: 176.,
        collapsed: 90.,
        priority: 4,
    },
    Group {
        expanded: 170.,
        compact: 118.,
        collapsed: 72.,
        priority: 5,
    },
];
pub fn layout(available: f32) -> Vec<Density> {
    let mut modes = vec![Density::Expanded; GROUPS.len()];
    let width = |modes: &[Density]| -> f32 {
        GROUPS
            .iter()
            .zip(modes)
            .map(|(g, d)| match d {
                Density::Expanded => g.expanded,
                Density::Compact => g.compact,
                Density::Collapsed => g.collapsed,
            })
            .sum::<f32>()
            + 24.
    };
    let mut order: Vec<_> = (0..GROUPS.len()).collect();
    order.sort_by_key(|i| std::cmp::Reverse(GROUPS[*i].priority));
    for target in [Density::Compact, Density::Collapsed] {
        for &i in &order {
            if width(&modes) <= available {
                return modes;
            }
            modes[i] = target;
        }
    }
    modes
}
pub fn width(modes: &[Density]) -> f32 {
    GROUPS
        .iter()
        .zip(modes)
        .map(|(g, d)| match d {
            Density::Expanded => g.expanded,
            Density::Compact => g.compact,
            Density::Collapsed => g.collapsed,
        })
        .sum::<f32>()
        + 24.
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ribbon_fits_supported_window_widths() {
        for w in [560., 640., 720., 800., 900., 1024., 1280., 1600., 1920.] {
            let m = layout(w);
            assert!(width(&m) <= w, "{w}: {}", width(&m));
        }
    }
    #[test]
    fn narrow_ribbon_preserves_tools_longest() {
        assert_eq!(layout(1600.), vec![Density::Expanded; 6]);
        assert_eq!(layout(560.), vec![Density::Collapsed; 6]);
        let m = layout(1100.);
        assert_eq!(m[1], Density::Expanded);
        assert_ne!(m[5], Density::Expanded);
    }
}
