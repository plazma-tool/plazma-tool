import React from 'react';
import { Column  } from 'bloomer';
import Slider from 'rc-slider';
import { numToStrPad, getFloatValuesFromCode } from './Helpers';

// Requires props:
// - code
// - onChangeLift
export class SliderColumns extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
    }

    onChangeLocal(newValue) {
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

// Requires props:
// - sliderValue: { name: "name", value: 0.0 }
// - onChangeLift
class PlazmaSlider extends React.Component {
    constructor(props) {
        super(props);
        this.onChangeLocal = this.onChangeLocal.bind(this);
    }

    onChangeLocal(x) {
        let newValue = {
            name: this.props.sliderValue.name,
            value: x,
        };
        this.props.onChangeLift(newValue);
    }

    render() {
        return (
            <div className="is-half">
              <span>{this.props.sliderValue.name}</span>
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

function getSliderValuesFromCode(code) {
    let re_slider = /float +([^ ]+) *= *([0-9.-]+); *\/\/ *!! slider *$/gm;
    return getFloatValuesFromCode(code, re_slider);
}

function replaceSliderValueInCode(newSliderValue, code) {
    const x = newSliderValue;
    let re_slider = new RegExp('(float ' + x.name + ' *= *)[0-9\\.]+(; *\\/\\/ *!! slider *$)', 'gm');
    let newCodeValue = code.replace(re_slider, '$1' + numToStrPad(x.value / 1000) + '$2');
    return newCodeValue;
}

