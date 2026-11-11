#!/usr/bin/env python3
"""Generate RMK startup defaults from the saved roBa Vial layout, fail closed."""
import argparse
import json
from pathlib import Path
import re
import tomllib

NAMES = {
 'NO':'No','TRNS':'_','TAB':'Tab','ESCAPE':'Escape','BSPACE':'Backspace',
 'ENTER':'Enter','SPACE':'Space','DELETE':'Delete','COMMA':'Comma','DOT':'Dot',
 'SLASH':'Slash','MINUS':'Minus','EQUAL':'Equal','SCOLON':'Semicolon','QUOTE':'Quote',
 'LBRACKET':'LeftBracket','RBRACKET':'RightBracket','NONUS_HASH':'NonusHash',
 'RO':'International1','JYEN':'International3','HENK':'International4','MHEN':'International5',
 'LALT':'LAlt','LGUI':'LGui','LCTRL':'LCtrl','LCTL':'LCtrl','LSHIFT':'LShift','LSFT':'LShift',
 'CAPSLOCK':'CapsLock','NUMLOCK':'NumLock','LEFT':'Left','RIGHT':'Right','UP':'Up','DOWN':'Down',
 'HOME':'Home','END':'End','PGUP':'PageUp','PGDOWN':'PageDown',
 'KP_PLUS':'KpPlus','KP_MINUS':'KpMinus','KP_ASTERISK':'KpAsterisk','KP_SLASH':'KpSlash',
 'WH_D':'MouseWheelDown','WH_U':'MouseWheelUp','WH_L':'MouseWheelLeft','WH_R':'MouseWheelRight',
 **{f'BTN{i}':f'MouseBtn{i}' for i in range(1,9)},
 **{f'KP_{i}':f'Kp{i}' for i in range(10)},
 **{str(i):f'Kc{i}' for i in range(10)},
}
MODS={'LGUI':'LGui','LCTL':'LCtrl','LSFT':'LShift','LALT':'LAlt','LCA':'LCtrl|LAlt','C_S':'LCtrl|LShift'}

def key(code):
    if code == -1: return 'No'  # Vial serializes undisplayed/unwired cells as -1.
    if not isinstance(code,str): raise ValueError(f'Unsupported key: {code!r}')
    if code.startswith('KC_'):
        k=code[3:]
        if re.fullmatch(r'[A-Z]|F(?:[1-9]|1[0-9]|2[0-4])',k):return k
        if k in NAMES:return NAMES[k]
    if re.fullmatch(r'TD\(\d+\)',code):return code
    if re.fullmatch(r'M\d+',code):return 'Macro'+code[1:]
    if re.fullmatch(r'USER\d+',code):return 'User'+str(int(code[4:]))
    if code in ('QK_BOOT','QK_REBOOT'):return {'QK_BOOT':'Bootloader','QK_REBOOT':'Reboot'}[code]
    m=re.fullmatch(r'LT(\d+)\((.+)\)',code)
    if m:return f'LT({m[1]},{key(m[2])})'
    m=re.fullmatch(r'([A-Z_]+)\((.+)\)',code)
    if m and m[1] in MODS:return f'WM({key(m[2])},{MODS[m[1]]})'
    raise ValueError(f'Unsupported Vial keycode: {code!r}')


def q(s):return json.dumps(s,ensure_ascii=False)

def render(source,template):
    if source.get('uid')!=int.from_bytes(bytes.fromhex('b9bc09b29d374cea'),'little'):
        raise ValueError('Unexpected Vial keyboard UID')
    layout=source['layout'];td=source['tap_dance'];macros=source['macro']
    if len(layout)!=8 or any(len(l)!=5 or any(len(r)!=11 for r in l) for l in layout):
        raise ValueError('Expected eight 5x11 layers')
    if len(td)!=32:raise ValueError('Expected 32 tap-dance entries')
    if any(any(c!='KC_NO' for c in row) for row in source.get('combo',[])):
        raise ValueError('Active combo conversion requires explicit support')
    if source.get('key_override') or source.get('alt_repeat_key'):
        raise ValueError('Active overrides require explicit support')
    config=tomllib.loads(template)
    mapping=[tuple(map(int,m)) for m in re.findall(r'\((\d+),(\d+)\)',config['layout']['matrix_map'])]
    start = template.find('# Startup defaults imported')
    if start < 0: start = re.search(r'(?m)^\[behavior(?:[.\]])', template).start()
    prefix=template[:start]
    aliases=template[template.index('[aliases]'):template.index('[[layer]]')]
    settings=source['settings']
    supported={'2','6','7','18','19','22','23','26','27'}
    if settings.keys()-supported:raise ValueError('Unrecognized behavior setting')
    if settings['18']!=20 or settings['19']!=20:raise ValueError('Tap intervals differ from the pinned RMK defaults')
    b=['# Startup defaults imported from config/261109_robaish.vil.',
       '[behavior.one_shot]',f'timeout = "{settings["6"]}ms"','',
       '[behavior.combo]',f'timeout = "{settings["2"]}ms"','combos = []','',
       '[behavior.morse]',f'hold_timeout = "{settings["7"]}ms"',
       f'prior_idle_time = "{settings["27"]}ms"',
       f'permissive_hold = {str(bool(settings["22"])).lower()}',
       f'hold_on_other_press = {str(bool(settings["23"])).lower()}',
       f'unilateral_tap = {str(bool(settings["26"])).lower()}','']
    for timeout in sorted({row[4] for row in td}):
        b += [f'[behavior.morse.profiles.vil_{timeout}]',f'hold_timeout = "{timeout}ms"',f'gap_timeout = "{timeout}ms"','']
    for row in td:
        tap,hold,double,after,timeout=row
        b+=['[[behavior.morse.morses]]',f'tap = {q(key(tap))}',f'hold = {q(key(hold))}',f'double_tap = {q(key(double))}',f'hold_after_tap = {q(key(after))}',f'profile = "vil_{timeout}"','']
    for macro in macros:
        ops=[]
        for op in macro:
            kind,*values=op
            if kind in ['tap','down','up']:
                for value in values:ops.append('{ operation = '+q(kind)+', keycode = '+q(key(value))+' }')
            elif kind=='text' and len(values)==1:ops.append('{ operation = "text", text = '+q(values[0])+' }')
            elif kind=='delay' and len(values)==1:ops.append('{ operation = "delay", duration = '+q(str(values[0])+'ms')+' }')
            else:raise ValueError(f'Unsupported macro operation {op!r}')
        b+=['[[behavior.macros.macros]]','operations = ['+', '.join(ops)+']','']
    layers=[]
    for i,layer in enumerate(layout):
        vals=[]
        for r,c in mapping:
            # All virtual gesture slots are deliberately unassigned; physical V
            # on layer 3 defaults to the assignable non-typing trigger.
            vals.append('No' if (r==4 or (r,c)==(0,5)) else 'User10' if (i,r,c)==(3,2,3) else key(layer[r][c]))
        # Format in the existing physical layout sequence plus the virtual row.
        widths=[10,12,12,9,4,4,4];lines=[];offset=0
        for n in widths:lines.append('  '.join(vals[offset:offset+n]));offset+=n
        if offset!=len(mapping):raise ValueError('Unexpected physical mapping')
        enc=source['encoder_layout'][i]
        if len(enc)!=1:raise ValueError('Expected exactly one encoder')
        layers+=['[[layer]]',f'name = {q(config["layer"][i]["name"])}','keys = """',*lines,'"""',f'encoders = [[{q(key(enc[0][1]))}, {q(key(enc[0][0]))}]]','']
    return prefix+'\n'.join(b)+'\n'+aliases+'\n'.join(layers)+'\n# Physical central, split peripheral, and synthetic gesture events may publish concurrently.\n[event.keyboard]\npubs = 3\nsubs = 4\n\n# Resolved live user actions drive the assignable gesture trigger.\n[event.action]\npubs = 1\nsubs = 1\n'

if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('source',type=Path);p.add_argument('keyboard',type=Path);p.add_argument('--check',action='store_true');a=p.parse_args()
    old=a.keyboard.read_text();new=render(json.loads(a.source.read_text()),old)
    if a.check:
        if old!=new:p.exit(1,'Default configuration is not synchronized with the saved layout\n')
    else:a.keyboard.write_text(new)
