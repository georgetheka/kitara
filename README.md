# Kitara

### Overview

Kitara is a simple cross-platform program written in Rust that can convert any MIDI guitar to a computer keyboard. I wrote this quickly on a Sunday afternoon, just for fun, so this program has only been tested on MacOs Catalina.

### Requirements

* Rust
* A MIDI-compatible guitar
* Ability to geek out

### Usage

After compiling with cargo, run the program as shown in its usage:

```
Usage: kitara <device-name> <path/to/config/csv>
```

where:
* `device-name` is the (approximate) name of the midi guitar device connected to your system
* `path/to/config/csv` is the path to `map.csv` (a default file is provided) that contains a custom
mapping between the fretboard and a computer keyboard


### Current mapping
I have included the following mapping which is essentially a form of QWERTY with
the left hand keys turned upside down. This allows easy playability with both hands by
tapping. 

The MIDI channels are specified on the first column and they can map to your own
string midi channels. Note that each guitar string must play in a separate channel, a feature
that all guitar midi players have. 


```
0	1	2	3	4	5	6	7	8	9	10	11	12	13	14	15	16	17	18	19	20	21	22	
--------------------------------------------------------------------------------------------
6|	RI	DO	LE		ES			BA	BA	BA	BA	BA				6	7	8	9	0	-	=	
5|		UP						b	v	c	x	z	SH			y	u	i	o	p	[	]	
4|								g	f	d	s	a				h	j	k	l	;			
3|								t	r	e	w	q				n	m	,	.	/	SH	SH	
2|	TA							5	4	3	2	1	`										
1|	TA							CT	AL	CM			SP	SP	SP	SP	SP	SP			EN	EN	
```

#### Key Abbreviations

* RI = Right Arrow
* DO = Down Arrow
* LE = Left Arrow
* UP = Up Arrow
* BA = Backspace
* TA = TAB
* CT = CTRL
* AL = ALT (Mac Option key)
* CM = CMD (Mac only)
* SP = Spacebar
* SH = Shift
* ES = Escape
* EN = Enter/Return

### Todo

* Add unit tests
* Refactor/optimize
* Support for any midi fretted instrument (maybe)
* Make tuning configurable (maybe)

