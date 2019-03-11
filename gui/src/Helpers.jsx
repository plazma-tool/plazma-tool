// @flow

// No index 0, to avoid == problems.
export const CurrentPage = {
    Settings: 1,
    Shaders: 2,
    Framebuffers: 3,
    QuadScenes: 4,
    PolygonScenes: 5,
    Images: 6,
    Models: 7,
    Timeline: 8,
    SyncTracks: 9,
    Library: 10,
};

export type ServerMsg = {
    data_type: string,
    data: string,
};

export type ContextData = {
    shader_sources: string[],
    frame_buffers: any,// FIXME FrameBuffer[],
    quad_scenes: any,// FIXME QuadScene[],
    polygon_scenes: any,// FIXME PolygonScenes[],
    polygon_context: any,// FIXME PolygonContext,
    sync_tracks_path: string,
    index: any,// FIXME DataIndex,
};

export type SceneBlock = {
    start: number,
    end: number,
    draw_ops: any,// FIXME DrawOp[],
};

export type TimeTrack = {
    scene_blocks: SceneBlock[],
};

export type Timeline = {
    tracks: TimeTrack[],
};

export type DmoData = {
    settings: {
        start_full_screen: bool,
        audio_play_on_start: bool,
        mouse_sensitivity: number,
        movement_sensitivity: number,
        total_length: number,
        [string]: mixed,
    },
    context: ContextData,
    timeline: Timeline,
};

export type InputEvent = SyntheticEvent<HTMLInputElement>;

export type Vec3 = [number, number, number];

export type SliderValue = {
    name: string,
    value: number,
};

export function getVec3ValuesFromCode(code: string = "", re: RegExp): Array<{name: string, vec: Vec3}> {
    let values: Array<{name: string, vec: Vec3}> = [];
    if (code.length === 0) {
        return values;
    }

    let match_vec3;
    while ((match_vec3 = re.exec(code)) !== null) {
        if (match_vec3 !== null) {
            let name = match_vec3[1].trim();
            let vec3_components = match_vec3[2].trim();
            let vec = [];

            let match_comp = vec3_components.match(/([0-9.-]+)/g);
            if (match_comp !== null) {
                match_comp.forEach((i) => {
                    let n = Number(i);
                    if (!isNaN(n)) {
                        vec.push(n);
                    }
                });
                if (vec.length === 3) {
                    let v = [vec[0], vec[1], vec[2]];
                    values.push({
                        name: name,
                        vec: v,
                    });
                }
            }
        }
    }

    return values;
}

export function getFloatValuesFromCode(code: string = "", re: RegExp): Array<SliderValue> {
    let values: Array<SliderValue> = [];
    if (code.length === 0) {
        return values;
    }

    let m;
    while ((m = re.exec(code)) !== null) {
        if (m !== null) {
            values.push({
                name: m[1].trim(),
                value: Math.floor(Number(m[2].trim()) * 1000),
            });
        }
    }

    return values;
}

export function numToStrPad(x: number): string {
    let s = x.toFixed(3).toString();
    if (s.indexOf('.') === -1) {
        return s + '.000';
    } else {
        return s.padEnd(5, '0');
    }
}

