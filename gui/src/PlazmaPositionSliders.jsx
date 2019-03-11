// @flow
import React from 'react';
import { Column  } from 'bloomer';
import Slider from 'rc-slider';
import { numToStrPad, getVec3ValuesFromCode } from './Helpers';

type XyzValue = { x: number, y: number, z: number };
type Position = { name: string, xyz: XyzValue };

type PSC_Props = {
    code: string,
    onChangeLift: (newCodeValue: string) => void,
};

export class PositionSlidersColumns extends React.Component<PSC_Props> {

    onChangeLocal = (newPositionValue: Position) => {
        let newCodeValue = replacePositionValueInCode(newPositionValue, this.props.code);
        this.props.onChangeLift(newCodeValue);
    }

    render() {
        let values = getPositionValuesFromCode(this.props.code);
        let sliders = values.map((position, idx) => {
            return (
                <PlazmaPositionSliders
                  key={position.name + idx}
                  position={position}
                  onChangeLift={this.onChangeLocal}
                />
            );
        });
        return (
            <Column>
              {sliders}
            </Column>
        );
    }
}

type PPS_Props = {
    position: Position,
    onChangeLift: (newPositionValue: Position) => void,
};

class PlazmaPositionSliders extends React.Component<PPS_Props> {
    render() {
        let p = this.props.position;
        return (
            <div className="is-half">
              <span>{p.name}</span>
              <PositionSliders
                position={p}
                onChangeLift={this.props.onChangeLift}
              />
            </div>
        );
    }
}

class PositionSliders extends React.Component<PPS_Props> {

    onChangeX = (x: number) => {
        let xyz = this.props.position.xyz;
        xyz.x = x;

        let newPositionValue: Position = {
            name: this.props.position.name,
            xyz: xyz,
        };
        this.props.onChangeLift(newPositionValue);
    }

    onChangeY = (y: number) => {
        let xyz = this.props.position.xyz;
        xyz.y = y;

        let newPositionValue = {
            name: this.props.position.name,
            xyz: xyz,
        };
        this.props.onChangeLift(newPositionValue);
    }

    onChangeZ = (z: number) => {
        let xyz = this.props.position.xyz;
        xyz.z = z;

        let newPositionValue = {
            name: this.props.position.name,
            xyz: xyz,
        };
        this.props.onChangeLift(newPositionValue);
    }

    render() {
        return (
            <div>
              <span>x</span>
              <Slider
                value={this.props.position.xyz.x}
                step={1}
                min={-1000}
                max={1000}
                onChange={this.onChangeX}
              />

              <span>y</span>
              <Slider
                value={this.props.position.xyz.y}
                step={1}
                min={-1000}
                max={1000}
                onChange={this.onChangeY}
              />

              <span>z</span>
              <Slider
                value={this.props.position.xyz.z}
                step={1}
                min={-1000}
                max={1000}
                onChange={this.onChangeZ}
              />
            </div>
        );
    }
}

export function xyzToVec3(pos: XyzValue): string {
    let vec = [ pos.x, pos.y, pos.z ].map((i) => {
        return numToStrPad(Number(i / 1000));
    });
    return 'vec3(' + vec[0] + ', ' + vec[1] + ', ' + vec[2] + ')';
}

function getPositionValuesFromCode(code: string): Position[] {
    let re_position = /vec3 +([^ ]+) *= *vec3\(([^)]+)\); *\/\/ *!! position *$/gm;
    let v = getVec3ValuesFromCode(code, re_position);
    let values: Position[] = v.map((val) => {
        let a: Position = {
            name: val.name,
            xyz: {
                x: Math.floor(val.vec[0] * 1000),
                y: Math.floor(val.vec[1] * 1000),
                z: Math.floor(val.vec[2] * 1000),
            }
        };
        return a;
    });
    return values;
}

function replacePositionValueInCode(newPositionValue: Position, code: string): string {
    const p = newPositionValue;
    let re_position = new RegExp('(vec3 +' + p.name + ' *= *)vec3\\([^\\)]+\\)(; *\\/\\/ *!! position *$)', 'gm');
    let newCodeValue = code.replace(re_position, '$1' + xyzToVec3(p.xyz) + '$2');
    return newCodeValue;
}

