"""Rasterize the user-approved SVG master; requires cairosvg."""
from pathlib import Path
import cairosvg
root=Path(__file__).resolve().parents[1]
mark=(root/'extension/static/brand/mark.svg').read_text().replace('currentColor','#111')
inner=mark[mark.index('<path'):mark.index('</svg>')]
icon=f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 768 768"><rect width="768" height="768" rx="144" fill="#fff"/><g transform="translate(64 124)">{inner}</g></svg>'
for size in [16,32,48,128]:
 cairosvg.svg2png(bytestring=icon.encode(),write_to=str(root/f'extension/static/brand/icon-{size}.png'),output_width=size,output_height=size)
