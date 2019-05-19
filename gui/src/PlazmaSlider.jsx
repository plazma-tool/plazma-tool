// @flow
import React from 'react';
import { Column  } from 'bloomer';
import Slider from 'rc-slider';
import { numToStrPad, getFloatValuesFromCode } from './Helpers';
import type { SliderValue } from './Helpers';

type SC_Props = {
    code: string,
    onChangeLift: (newCodeValue: string) => void,
};

export class SliderColumns extends React.Component<SC_Props> {

    onChangeLocal = (newValue: SliderValue) => {
        let newCodeValue = replaceSliderValueInCode(newValue, this.props.code);
        this.props.onChangeLift(newCodeValue);
    }

    render() {
        let values = getSliderValuesFromCode(this.props.code);
        let sliders = values.map((value, idx) => {
            return (
                <PlazmaSlider
                  key={value.name + idx}
                  sliderValue={value}
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

type PS_Props = {
    sliderValue: SliderValue,
    onChangeLift: (newValue: SliderValue) => void,
};

class PlazmaSlider extends React.Component<PS_Props> {

    onChangeLocal = (x: number) => {
        let newValue: SliderValue = {
            name: this.props.sliderValue.name,
            line_number: this.props.sliderValue.line_number,
            value: x,
        };
        this.props.onChangeLift(newValue);
    }

    render() {
        return (
            <div className="is-half">
              <span>{this.props.sliderValue.name} L{this.props.sliderValue.line_number + 1}</span>
              <Slider
                value={this.props.sliderValue.value}
                step={1}
                min={0}
                max={1000}
                onChange={this.onChangeLocal}
              />
            </div>
        );
    }
}

function getSliderValuesFromCode(code: string): Array<SliderValue> {
    let re_slider = /float +([^ ]+) *= *([0-9.-]+); *\/\/ +ui_slider *$/gm;
    return getFloatValuesFromCode(code, re_slider);
}

function replaceSliderValueInCode(newSliderValue: SliderValue, code: string): string {
    const x = newSliderValue;
    let re_slider = new RegExp('(float ' + x.name + ' *= *)[0-9\\.]+(; *\\/\\/ +ui_slider *$)', 'gm');
    let lines = code.split("\n");
    lines[x.line_number] = lines[x.line_number].replace(re_slider, '$1' + numToStrPad(x.value / 1000) + '$2');
    let newCodeValue = lines.join("\n");
    return newCodeValue;
}

