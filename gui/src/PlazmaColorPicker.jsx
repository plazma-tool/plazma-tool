// @flow
import React from 'react';
import { Columns, Column  } from 'bloomer';
import { SketchPicker } from 'react-color';
import { numToStrPad, getVec3ValuesFromCode } from './Helpers';
import type { InputEvent } from './Helpers';

type CPC_Props = {
    code: string,
    onChangeLift: (newCodeValue: string) => void,
};

export class ColorPickerColumns extends React.Component<CPC_Props> {
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

type RgbaValue = { r: number, g: number, b: number, a: number };
type RgbValue = { r: number, g: number, b: number };

type Color = { name: string, rgba: RgbaValue };
type SketchPickerColor = { name: string, rgb: { r: number, g: number, b: number } };

type PCP_Props = {
    code: string,
    onChangeLift: (newCodeValue: string) => void,
    color: Color,
};

class PlazmaColorPicker extends React.Component<PCP_Props> {

    onChangeLocal = (newColorValue: SketchPickerColor) => {
        let newCodeValue = replaceColorValueInCode(newColorValue, this.props.code);
        this.props.onChangeLift(newCodeValue);
    }

    onChangeColor = (color: SketchPickerColor, event: InputEvent) => {
        let c: Color = this.props.color;
        let newColorValue: SketchPickerColor = {
            name: c.name,
            rgb: color.rgb,
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

function rgbToVec3(col: RgbValue | RgbaValue): string {
    let vec = [ col.r, col.g, col.b ].map((i) => {
        return numToStrPad(Number((i / 255)));
    });
    return 'vec3(' + vec[0] + ', ' + vec[1] + ', ' + vec[2] + ')';
}

function replaceColorValueInCode(newColorValue: SketchPickerColor, code: string): string {
    const c = newColorValue;
    let re_color = new RegExp('(vec3 +' + c.name + ' *= *)vec3\\([^\\)]+\\)(; *\\/\\/ *!! color *$)', 'gm');
    let newCodeValue = code.replace(re_color, '$1' + rgbToVec3(c.rgb) + '$2');
    return newCodeValue;
}

function getColorValuesFromCode(code: string): Color[] {
    let re_color = /vec3 +([^ ]+) *= *vec3\(([^)]+)\); *\/\/ *!! color *$/gm;
    let v = getVec3ValuesFromCode(code, re_color);
    let values = v.map((val) => {
        let c: Color = {
            name: val.name,
            rgba: {
                r: Math.floor(val.vec[0] * 255),
                g: Math.floor(val.vec[1] * 255),
                b: Math.floor(val.vec[2] * 255),
                a: 1.0,
            }
        };
        return c;
    });
    return values;
}
