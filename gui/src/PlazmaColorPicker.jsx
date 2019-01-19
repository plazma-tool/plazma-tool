import React from 'react';
import { Columns, Column  } from 'bloomer';
import { SketchPicker } from 'react-color';
import { numToStrPad, getVec3ValuesFromCode } from './Helpers';

// Requires props:
// - code
// - onChangeLift
export class ColorPickerColumns extends React.Component {
    render() {
        let values = getColorValuesFromCode(this.props.code);
        let pickers = values.map((color, idx) => {
            return (
                <PlazmaColorPicker
                  key={color.name + idx}
                  code={this.props.code}
                  color={color}
                  onChangeLift={this.props.onChangeLift}
                />
            );
        });
        return (
            <Column>
              <Columns>
                {pickers}
              </Columns>
            </Column>
        );
    }
}

// Requires props:
// - code
// - color: { name: "name", rgba: { r: 0, g: 0, b: 0, a: 0 } }
// - onChangeLift
class PlazmaColorPicker extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
        this.onChangeColor = this.onChangeColor.bind(this);
    }

    onChangeLocal(newColorValue) {
        let newCodeValue = replaceColorValueInCode(newColorValue, this.props.code);
        this.props.onChangeLift(newCodeValue);
    }

    onChangeColor(color, event) {
        let c = this.props.color;
        let newColorValue = {
            name: c.name,
            rgba: color.rgb,
        };
        this.onChangeLocal(newColorValue);
    }

    render() {
        let c = this.props.color;
        return (
            <div className="is-one-quarter">
              <span>{c.name}</span>
              <SketchPicker
                color={c.rgba}
                onChange={this.onChangeColor}
              />
            </div>
        );
    }
}

function rgbToVec3(col) {
    let vec = [ col.r, col.g, col.b ].map((i) => {
        return numToStrPad(Number((i / 255)));
    });
    return 'vec3(' + vec[0] + ', ' + vec[1] + ', ' + vec[2] + ')';
}

function replaceColorValueInCode(newColorValue, code) {
    const c = newColorValue;
    let re_color = new RegExp('(vec3 +' + c.name + ' *= *)vec3\\([^\\)]+\\)(; *\\/\\/ *!! color *$)', 'gm');
    let newCodeValue = code.replace(re_color, '$1' + rgbToVec3(c.rgba) + '$2');
    return newCodeValue;
}

function getColorValuesFromCode(code) {
    let re_color = /vec3 +([^ ]+) *= *vec3\(([^\)]+)\); *\/\/ *!! color *$/gm;
    let v = getVec3ValuesFromCode(code, re_color);
    let values = v.map((val) => {
        return {
            name: val.name,
            rgba: {
                r: Math.floor(val.vec[0] * 255),
                g: Math.floor(val.vec[1] * 255),
                b: Math.floor(val.vec[2] * 255),
                a: 1.0,
            }
        };
    });
    return values;
}
