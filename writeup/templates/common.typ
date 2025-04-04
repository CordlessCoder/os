#import "theme.typ": *
#import "@preview/cetz:0.3.3"
#let canvas(length: 1cm, debug: false, background: none, body) = context {
    with_cur_theme(theme => {
        // set box(fill: theme.overlay,  radius: 2pt, inset: 5pt)
        cetz.canvas(length: length, debug: debug, background: background, {
            cetz.draw.set-style(
                content: (padding: .2, fill: theme.overlay, stroke: none),
                fill: theme.surface,
                mark: (stroke: theme.text),
                line: (stroke: theme.text),
            )
            body
        }) 
    })
}
#import "@preview/tablex:0.0.9": tablex as tbx, cellx as cell, colspanx as colspan, rowspanx as rowspan, vlinex as vline, hlinex as hline, gridx
#import footnote as footnote_impl
#import "@preview/pinit:0.2.2": pin, pinit-arrow, pinit, pinit-place, pinit-highlight, pinit-line, pinit-line-to, pinit-point-from, pinit-point-to, pinit-rect
#let pi-point-from(..args) = with_cur_theme(
    theme => pinit-point-from(fill: theme.text, ..args)
)
#let pi-point-to(..args) = with_cur_theme(
    theme => pinit-point-to(fill: theme.text, ..args)
)
#let pi-line-from(..args) = with_cur_theme(
    theme => pinit-line-from(fill: theme.text, ..args)
)
#let pi-line-to(..args) = with_cur_theme(
    theme => pinit-line-to(fill: theme.text, ..args)
)
#let pi-highlight(..args) = with_cur_theme(
    theme => pinit-highlight(fill: theme.surface, ..args)
)
#let pi-rect(..args) = with_cur_theme(
    theme => pinit-rect(fill: theme.text, ..args)
)
#let pi-arrow(..args) = with_cur_theme(
    theme => pinit-arrow(fill: theme.text, ..args)
)

// Automatically add full stops to unterminated footnotes
#let eblock(body, ..args) = {
    with_cur_theme(theme => 
        block(
            breakable: args.named().at("breakable", default: true),
            fill: args.named().at(default: theme.overlay, "bg"),
            radius: 0.3em, 
            stroke: theme.subtle,
            inset: 1em,
            columns(args.named().at(default: 1, "c"),
                text(fill: args.named().at(default: theme.text, "fg"),
                    body
                )
            )
        )
    )
}
#let gr(body) = {
    with_cur_theme(theme => 
        text(
            fill: theme.subtle,
            body
        )
    )
}
#let TODO(body) = {
    with_cur_theme(theme => 
        text(fill: theme.love, [TODO: ] + body)
    )
}
#let footnote(content, ..args) = {
    let get_text(content) = {
        let children = content.fields().at("children", default: none)
        if children == none {
        children = (content,)
    }
        let last = children.last().fields().at("body", default: none)
        if last == none {
        last = children.last()
    }
        return last.fields().at("text", default: "")
    }
    let text = get_text(content)
    let terminated = text.ends-with(regex("(\.|,|!|;|:|…)"))
    footnote_impl(content + "." * (int(not terminated)), ..args)
}
#let longdiv(divident, divisor, result, operations, inset: 2pt, width: 1pt) = {
    locate(l => {
        set box(inset: inset, stroke: current_theme.at(l).text)
        grid(
            columns: 2,
            row-gutter: 2pt, 
            [],
            box(result, stroke: none),
            box(divisor, stroke: none),
            [
            // // This wraps everything in one box
            // #box(stroke: (top: width, left: width, right: none, bottom: none))[
            //#divident\
            //#operations
            //]

            // This only wraps the dividend in a box
            #box(stroke: (top: width, left: width, right: none, bottom: none))[
                #divident
            ]\
            #box(stroke: none, inset: (top: 0pt))[
                #operations
            ]\
        ]
        )
    })
}
#let ulm(lw: 1pt, content) = {
    content
    style(s => {
        let width = measure($content$, s).width;
        place(
            dx: -width,
            box(inset: 0pt, outset: 2pt, baseline: 3pt, stroke: (bottom: lw, top: none, right:none, left: none), width: width
            )[
            ]
        )})
}
#let olm(lw: 1pt, content) = {
    content
    style(s => {
        let width = measure($content$, s).width;
        place(
            dx: -width,
            dy: -6pt,
            box(inset: 0pt, outset: 2pt, baseline: 0pt, stroke: (top: lw, bottom: none, right:none, left: none), width: width
            )[
            ]
        )})
}
#let nl(content) = {
    pad(top: 1pt)[#box(inset: 0pt, stroke: none, $content$)]
}
#let sup(body, size: 1.2em, origin: left, ..args) = {
    // scale(origin: origin,..args, amount, body)
    set text(size: size)
    body
}
#let hl(body) = {
    highlight(body)
}
#let mass_number(top, bottom, body) = {
    $upright(#[#body])^top_bottom$
}
// #let plotthis(size: (7, 5), dimm: (0, 1, 0, 1), callback) = {
//   with_cur_theme(theme => {
//     canvas({
//     cetz.plot.draw.set-style(
//       stroke: theme.subtle,
//       tick: (stroke: theme.subtle),
//       mark: (stroke: theme.subtle),
//       grid: (stroke: (paint: theme.subtle)),
//       line: (stroke: theme.pine),
//     )
//     let config = cetz.plot.plot.with(
//       size: size,
//       y-min: dimm.at(2),
//       y-max: dimm.at(3),
//       x-min: dimm.at(0),
//       x-max: dimm.at(1),
//     )
//     let coord_map(x, y) = {
//       let scale_y(n) = n/(dimm.at(3) - dimm.at(2))*size.at(1)
//       let scale_x(n) = n/(dimm.at(1) - dimm.at(0))*size.at(0)
//       (scale_x(x - dimm.at(0)),
//       scale_y(y - dimm.at(2)))
//     }
//     let x = dimm.slice(0, 2)
//     let y = dimm.slice(2, 2)
//     callback(config, (to_plot: coord_map, dimm: dimm, size: size, width: x, length: y))
//   })
// })
// }
