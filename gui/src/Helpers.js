export const CurrentPage = {
    Settings: 1,
    ContextShader: 2,
    Timeline: 3,
};

export function getVec3ValuesFromCode(code, re) {
    let values = [];
    if (code === null) {
        return values;
    }

    let match_vec3 = null;
    while ((match_vec3 = re.exec(code)) !== null) {
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
                values.push({
                    name: name,
                    vec: vec,
                });
            }
        }
    }

    return values;
}

export function getFloatValuesFromCode(code, re) {
    let values = [];
    if (code === null) {
        return values;
    }

    let m = null;
    while ((m = re.exec(code)) !== null) {
        values.push({
            name: m[1].trim(),
            value: Math.floor(Number(m[2].trim()) * 1000),
        });
    }

    return values;
}

export function numToStrPad(x) {
    let s = x.toFixed(3).toString();
    if (s.indexOf('.') === -1) {
        return s + '.000';
    } else {
        return s.padEnd(5, '0');
    }
}

