// @flow

// No index 0, to avoid == problems.
export const CurrentPage = {
    Settings: 1,
    Shaders: 2,
    Framebuffers: 3,
    QuadScenes: 4,
    PolygonScenes: 5,
    Models: 6,
    Timeline: 7,
    SyncTracks: 8,
    Library: 9,
};

export const EditorsLayout = {
    OneMax: 1,
    TwoVertical: 2,
    TwoHorizontal: 3,
    ThreeMainLeft: 4,
    ThreeMainRight: 5,
    ThreeMainTop: 6,
    ThreeMainBottom: 7,
    FourEven: 8,
};

export type Editor = {
    source_idx: number,
};

export type ShaderEditors = {
    layout: number,
    full_height: number,
    current_editor_idx: number,
    editors: Editor[],
};

export type ShaderErrorData = {
    text: string,
    id: number,
};

export type Shader = {
    content: string,
    file_path: string,
    source_idx: number,
    error_data: ?ShaderErrorData,
    prev_error_data: ?ShaderErrorData,
    decoration_ids: [],
    saved_view_state: ?{},
};

export type ShaderErrorMessage = {
    file_index: number,
    line_number: number,
    error_type: string,
    error_id: string,
    error_text: string,
};

export type ServerMsg = {
    data_type: string,
    data: string,
};

export type PixelFormat = "NOOP" | "RED_u8" | "RGB_u8" | "RGBA_u8";

export type BufferKind = "NOOP" | "Empty_Texture" | "Image_Texture";

export type FrameBuffer = {
    name: string,
    kind: BufferKind,
    format: PixelFormat,
    image_path: string,
};

export type BuiltIn =
    | "Time"
    | "Window_Width"
    | "Window_Height"
    | "Screen_Width"
    | "Screen_Height"
    | "Camera_Pos_X"
    | "Camera_Pos_Y"
    | "Camera_Pos_Z"
    | "Camera_Front_X"
    | "Camera_Front_Y"
    | "Camera_Front_Z"
    | "Camera_Up_X"
    | "Camera_Up_Y"
    | "Camera_Up_Z"
    | "Camera_LookAt_X"
    | "Camera_LookAt_Y"
    | "Camera_LookAt_Z"
    | "Fovy"
    | "Znear"
    | "Zfar"
    | "Light_Pos_X"
    | "Light_Pos_Y"
    | "Light_Pos_Z"
    | "Light_Dir_X"
    | "Light_Dir_Y"
    | "Light_Dir_Z"
    | "Light_Strength"
    | "Light_Constant_Falloff"
    | "Light_Linear_Falloff"
    | "Light_Quadratic_Falloff"
    | "Light_Cutoff_Angle"
    | [ string ];

export type UniformMapping =
    | "NOOP"
    | { Float: [number, BuiltIn] }
    | { Vec2: [number, BuiltIn, BuiltIn] }
    | { Vec3: [number, BuiltIn, BuiltIn, BuiltIn] }
    | { Vec4: [number, BuiltIn, BuiltIn, BuiltIn, BuiltIn] };

export type BufferMapping =
    | "NOOP"
    | { Sampler2D: [number, string] };

export type QuadScene = {
    name: string,
    vert_src_path: string,
    frag_src_path: string,
    layout_to_vars: UniformMapping[],
    binding_to_buffers: BufferMapping[]
};

export type SceneObject = {
    name: string,
    position: string,// ValueVec3, TODO union
    euler_rotation: string,// ValueVec3, TODO
    scale: string,// ValueFloat, TODO
    layout_to_vars: any[],// UniformMapping[], TODO
    binding_to_buffers: any[],// BufferMapping[], TODO
};

export type PolygonScene = {
    name: string,
    scene_objets: SceneObject[],
};

export type ModelType = "NOOP" | "Cube" | "Obj";

export type Model = {
    name: string,
    model_type: ModelType,
    vert_src_path: string,
    frag_src_path: string,
    obj_path: string,
};

export type PolygonContext = {
    models: Model[],
};

// BTreeMap<String, usize>
export type NameToIdxMap = { [string]: number };

export type DataIndex = {
    shader_paths: string[],
    shader_path_to_idx: NameToIdxMap,

    image_paths: string[],
    image_path_to_idx: NameToIdxMap,
    image_path_to_format: { [string]: PixelFormat },

    obj_paths: string[],
    quad_scene_name_to_idx: NameToIdxMap,
    polygon_scene_name_to_idx: NameToIdxMap,
    model_name_to_idx: NameToIdxMap,
    obj_path_to_idx: NameToIdxMap,
    buffer_name_to_idx: NameToIdxMap,
};

export type ContextData = {
    shader_sources: string[],
    frame_buffers: FrameBuffer[],
    quad_scenes: QuadScene[],
    polygon_scenes: PolygonScene[],
    polygon_context: PolygonContext,
    sync_tracks_path: string,
    index: DataIndex,
};

export type SceneBlock = {
    start: number,
    end: number,
    draw_ops: string[],// DrawOp[], TODO enum
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
    line_number: number,
    value: number,
};

export function getVec3ValuesFromCode(code: string = "", re: RegExp): Array<{name: string, line_number: number, vec: Vec3}> {
    let values: Array<{name: string, line_number: number, vec: Vec3}> = [];
    if (code.length === 0) {
        return values;
    }

    let lines: Array<string> = code.split("\n");
    lines.forEach((line, idx) => {
        let match_vec3;
        while ((match_vec3 = re.exec(line)) !== null) {
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
                            line_number: idx,
                            vec: v,
                        });
                    }
                }
            }
        }
    });

    return values;
}

export function getFloatValuesFromCode(code: string = "", re: RegExp): Array<SliderValue> {
    let values: Array<SliderValue> = [];
    if (code.length === 0) {
        return values;
    }

    let lines: Array<string> = code.split("\n");
    lines.forEach((line, idx) => {
        let m;
        while ((m = re.exec(line)) !== null) {
            if (m !== null) {
                values.push({
                    name: m[1].trim(),
                    line_number: idx,
                    value: Math.floor(Number(m[2].trim()) * 1000),
                });
            }
        }
    });

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

export function parseShaderErrorText(msg: string): ShaderErrorMessage[] {
    let results: ShaderErrorMessage[] = [];

    /*
    0(10) : warning C7022: unrecognized profile specifier "l"
    0(10) : error C0502: syntax error at token "l"

    0(10) : warning C7022: unrecognized profile specifier "l"
    0
    10
    warning
    C7022
    unrecognized profile specifier "l"

    0(10) : error C0502: syntax error at token "l"
    0
    10
    error
    C0502
    syntax error at token "l"
    */

    let re = /^(\d+)\((\d+)\) *: *(\w+) *(\w+) *: *(.*)$/gm;
    let match_error;
    while ((match_error = re.exec(msg)) !== null) {
        if (match_error !== null) {
            let file_index = 0;
            let n = Number(match_error[1].trim());
            if (!isNaN(n)) {
                file_index = n;
            }

            n = Number(match_error[2].trim());
            let line_number = 0;
            if (!isNaN(n)) {
                line_number = n;
            }

            let error_type = match_error[3].trim();
            let error_id   = match_error[4].trim();
            let error_text = match_error[5].trim();

            results.push({
                file_index: file_index,
                line_number: line_number,
                error_type: error_type,
                error_id: error_id,
                error_text: error_text,
            });
        }
    }

    return results;
}

/*
function getShaderIndex(dmoData: DmoData, selectedPath: string) {
    if (dmoData === null) {
        return 0;
    }
    let idx = dmoData.context.index.shader_path_to_idx[selectedPath];
    if (idx === null) {
        console.log("Error: selectedPath not found in shaders");
        return 0;
    }
    let n = Number(idx);
    if (!isNaN(n)) {
        return n;
    } else {
        console.log("Error, index is not a number");
        return 0;
    }
}
*/

export function pathBasename(path: string) {
    return path.replace(/.*\//, '')
}

