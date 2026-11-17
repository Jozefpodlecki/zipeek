use yew::prelude::*;

const SILHOUETTES: [&str; 12] = [
    "M24 113 Q9 109 13 95 Q16 82 35 78 Q29 58 48 48 Q68 36 87 48 Q98 23 124 22 Q150 20 163 44 Q183 32 202 44 Q215 16 240 27 Q261 37 258 62 Q284 63 295 83 Q312 111 283 120 Q255 122 228 118 Q200 114 174 123 Q147 130 121 121 Q96 113 70 123 Q44 126 24 113 Z",
    "M20 111 Q7 102 20 91 Q31 82 51 85 Q49 67 69 63 Q82 60 94 71 Q111 52 132 57 Q147 28 169 24 Q196 20 211 48 Q231 43 244 60 Q253 73 253 85 Q283 77 297 95 Q310 120 270 121 Q245 127 222 120 Q195 111 170 124 Q143 129 118 119 Q90 112 65 123 Q37 130 20 111 Z",
    "M17 108 Q6 95 23 87 Q37 80 52 85 Q61 72 78 75 Q85 49 107 52 Q129 54 134 70 Q157 39 182 47 Q205 52 207 74 Q230 64 247 78 Q270 73 286 87 Q307 109 277 118 Q247 123 222 117 Q195 111 166 120 Q135 132 109 120 Q79 113 54 120 Q29 121 17 108 Z",
    "M28 112 Q13 103 22 88 Q32 72 52 77 Q44 55 61 39 Q84 19 110 34 Q122 8 152 10 Q176 11 186 38 Q211 30 225 51 Q234 63 231 79 Q256 67 275 82 Q305 104 276 122 Q248 129 226 119 Q202 113 175 126 Q148 135 119 123 Q88 112 62 125 Q40 130 28 112 Z",
    "M18 110 Q3 93 25 86 Q45 80 59 85 Q68 64 89 69 Q106 74 109 83 Q123 58 143 60 Q158 40 181 48 Q204 54 204 77 Q223 66 244 76 Q262 66 282 79 Q316 98 289 118 Q264 122 239 117 Q206 109 182 124 Q156 135 129 119 Q99 110 76 124 Q40 130 18 110 Z",
    "M21 112 Q4 100 19 85 Q31 74 47 81 Q50 49 74 46 Q88 42 103 56 Q109 27 130 28 Q153 29 155 53 Q165 18 192 19 Q215 21 223 50 Q239 45 251 58 Q258 68 256 81 Q287 75 298 96 Q307 119 277 122 Q250 127 226 119 Q202 110 178 126 Q148 135 117 122 Q90 112 65 126 Q34 130 21 112 Z",
    "M23 111 Q5 101 18 86 Q33 69 55 81 Q65 69 77 72 Q83 40 111 39 Q125 39 135 50 Q143 35 159 34 Q179 33 189 48 Q196 28 219 34 Q243 42 243 64 Q256 64 266 75 Q286 72 299 91 Q312 118 279 123 Q249 126 222 120 Q194 115 163 126 Q135 139 102 123 Q75 116 49 126 Q30 127 23 111 Z",
    "M28 112 Q9 108 16 91 Q23 72 44 75 Q58 54 77 66 Q81 35 105 28 Q134 20 149 45 Q160 15 188 23 Q207 28 215 53 Q232 45 247 59 Q259 71 255 84 Q283 73 299 94 Q313 116 280 122 Q253 124 230 117 Q206 110 176 127 Q141 137 115 122 Q86 114 61 125 Q37 126 28 112 Z",
    "M17 113 Q0 101 18 91 Q30 82 47 86 Q56 74 72 76 Q76 54 92 51 Q112 45 124 62 Q142 30 168 31 Q189 33 198 57 Q223 50 239 69 Q258 66 272 83 Q291 81 304 102 Q307 119 283 123 Q253 119 233 124 Q209 134 181 123 Q148 113 122 128 Q96 138 71 124 Q39 131 17 113 Z",
    "M20 110 Q7 96 27 85 Q44 79 59 84 Q69 44 99 50 Q117 32 138 47 Q154 22 177 31 Q196 37 202 58 Q223 47 240 68 Q267 60 287 85 Q307 110 280 119 Q257 126 228 121 Q198 118 174 127 Q142 136 114 124 Q86 111 60 120 Q37 126 20 110 Z",
    "M26 116 Q6 112 15 96 Q22 82 43 85 Q45 68 64 64 Q79 63 91 76 Q100 27 126 28 Q152 25 166 55 Q177 36 194 44 Q209 48 212 70 Q240 57 259 75 Q281 73 295 91 Q311 115 279 124 Q257 127 232 119 Q207 113 181 126 Q148 135 121 123 Q91 118 66 128 Q42 135 26 116 Z",
    "M15 108 Q2 92 26 84 Q47 76 63 88 Q74 69 93 75 Q104 44 131 44 Q152 48 161 66 Q172 24 196 26 Q223 28 225 60 Q250 57 263 77 Q289 70 301 96 Q311 118 285 122 Q249 127 219 115 Q196 106 172 121 Q141 133 111 121 Q80 109 56 125 Q32 125 15 108 Z",
];

pub(super) fn view(variant: usize, gradient_id: &str) -> Html {
    let outline = SILHOUETTES[variant % SILHOUETTES.len()];
    let highlight = match variant % 4 {
        0 => "M42 87 Q49 53 76 57 Q88 40 107 49 Q122 35 139 47 Q99 83 42 87 Z",
        1 => "M57 84 Q66 68 83 73 Q118 52 141 68 Q110 85 57 84 Z",
        2 => "M74 84 Q88 53 112 59 Q130 67 146 61 Q130 91 74 84 Z",
        _ => "M55 85 Q65 58 90 57 Q106 42 123 55 Q112 81 55 85 Z",
    };
    let gradient_fill = format!("url(#{gradient_id})");
    html! {
        <svg
            class="pointer-events-none absolute inset-0 h-full w-full overflow-visible drop-shadow-[0_12px_12px_rgba(5,25,49,.20)] transition-[filter] duration-250 ease-out motion-reduce:transition-none group-hover:drop-shadow-[0_0_10px_rgba(156,215,255,.55)] group-focus-visible:drop-shadow-[0_0_10px_rgba(156,215,255,.55)] group-data-[cloud-hint=true]:drop-shadow-[0_0_17px_rgba(54,173,255,.8)] group-aria-pressed:drop-shadow-[0_0_12px_rgba(87,190,255,.65)] group-data-[cloud-incorrect=true]:drop-shadow-[0_0_13px_rgba(238,93,122,.72)]"
            viewBox="0 0 320 148"
            aria-hidden="true"
            focusable="false"
        >
            <defs>
                <linearGradient
                    id={gradient_id.to_owned()}
                    x1="0%"
                    y1="0%"
                    x2="20%"
                    y2="100%"
                >
                    <stop offset="0%" stop-color="#ffffff" />
                    <stop offset="48%" stop-color="#eff7ff" />
                    <stop offset="100%" stop-color="#b8d3e9" />
                </linearGradient>
            </defs>
            <path
                class="stroke-[#b7d2e8] stroke-[1.5px] transition-[fill,stroke,stroke-width] duration-250 ease-out motion-reduce:transition-none group-data-[cloud-incorrect=true]:stroke-[#e96b83] group-data-[cloud-incorrect=true]:stroke-[3px] group-data-[cloud-hint=true]:stroke-[#3b9ce8] group-data-[cloud-hint=true]:stroke-[3px] group-aria-pressed:stroke-[#67b9ee] group-aria-pressed:stroke-[3px] contrast-more:fill-white contrast-more:stroke-[#244d70]"
                d={outline}
                fill={gradient_fill}
            />
            <path
                class="fill-[#a7dbff] opacity-0 transition-opacity duration-250 ease-out motion-reduce:transition-none group-data-[cloud-hint=true]:opacity-40 group-aria-pressed:opacity-80"
                d={outline}
            />
            <path
                class="fill-[#b6ffdf] opacity-0 transition-opacity duration-300 ease-out motion-reduce:transition-none group-data-[cloud-success=true]:opacity-90"
                d={outline}
            />
            <path
                class="fill-[#ffbdcb] opacity-0 transition-opacity duration-200 ease-out motion-reduce:transition-none group-data-[cloud-incorrect=true]:opacity-85"
                d={outline}
            />
            <path
                class="fill-white opacity-70 transition-opacity duration-250 ease-out motion-reduce:transition-none group-aria-pressed:opacity-90 group-data-[cloud-incorrect=true]:opacity-45"
                d={highlight}
            />
        </svg>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_contours_have_independent_geometry() {
        let lower_segments = SILHOUETTES
            .iter()
            .map(|path| {
                path.rsplit_once(" Q")
                    .unwrap()
                    .0
                    .rsplit_once(" Q")
                    .unwrap()
                    .1
            })
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(lower_segments.len(), 12);
    }

    #[test]
    fn eight_distinct_silhouettes_are_available() {
        let distinct = SILHOUETTES
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(distinct.len(), 8);
        assert!(
            SILHOUETTES
                .iter()
                .all(|path| path.starts_with('M') && path.ends_with('Z'))
        );
    }
}
