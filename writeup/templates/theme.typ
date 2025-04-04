#let default_theme = {
    import "themes/light.typ": theme
    theme
}
#let current_theme = state("theme", default_theme)
#let with_cur_theme(cb) = context {
    let theme = current_theme.get()
    cb(theme)
}
#let doc(
    theme: "light", 
    header: [_Roman Moisieiev_],
    attributed: [*6 Condell, Thomond Community College*],
    footer: [],
    heading_numbering: 0,
    pre_outline: [],
    outline_title: [Contents],
    outline_depth: 2,
    auto_outline: false,
    page_margin: (x: 3em, y: 5em),
    font: "Libertinus Serif",
    page_numbering: "1",
    size: 11pt
) = {
    let theme = {
        import "themes/" + theme + ".typ": theme
        theme
    }

    let apply_theme(content, theme_data) = [
        #set par(linebreaks: "optimized", justify: true)
        #set page(
            header: [
            #header
            #h(1fr)
            #attributed
        ],
            margin: page_margin,
            footer: [#footer #h(1fr) #if page_numbering != none {
            context counter(page).display(page_numbering)
        }],
            numbering: "1",
            fill: theme_data.base,
        )
        #set text(fill: theme_data.text, font: font, size: size)
        #set table(fill: theme_data.overlay, stroke: theme_data.text)
        #set circle(stroke: theme_data.subtle, fill: theme_data.overlay)
        #set ellipse(stroke: theme_data.subtle, fill: theme_data.overlay)
        #set line(stroke: theme_data.subtle)
        #set curve(stroke: theme_data.subtle, fill: theme_data.overlay)
        #set polygon(stroke: theme_data.subtle, fill: theme_data.overlay)
        #set rect(stroke: theme_data.subtle, fill: theme_data.overlay)
        #set square(stroke: theme_data.subtle, fill: theme_data.overlay)
        #set highlight(fill: theme.highlight.low)
        #show link: set text(fill: theme_data.iris)
        #show ref: set text(fill: theme_data.foam)
        #show footnote: set text(fill: theme_data.pine)
        #set colbreak(weak: true)
        #set pagebreak(weak: true)
        #show raw: set text(font: "Fira Code")
        #set raw(tab-size: 4)
        #if theme.at("code_theme", default: none) != none {
        set raw(theme: "themes/" + theme.code_theme)
    }
        #show raw: it => {
            text(font: "JetBrainsMono NF", it)
        }
        #import "@preview/quick-maths:0.2.1": shorthands

        #show: shorthands.with(
            ($+-$, $plus.minus$),
            ($~=$, $tilde.eq$),
        )

        #current_theme.update(theme_data)

        #content
    ]
    let apply(content) = {
        apply_theme(content, theme)
    }


    if heading_numbering == 0 {
    heading_numbering = (..n) => {
    let n = n.pos();
    let out
    for i in range(0, n.len()) {
    if calc.odd(i) {
    out = $#out^#n.at(i)$
} else {
    out = $#out #numbering("I", n.at(i))$
}
}
    out
}
}
    let doc(content) = {
        show: apply
        set heading(numbering: heading_numbering)
        if auto_outline {
        page()[
            #[
            #set heading(outlined: false, numbering: none)
            #pre_outline
        ]
            #outline(
                indent: auto,
                target: heading.where(outlined: true),
                title: outline_title,
                depth: outline_depth
            )
        ]
    }
        content
    }
    return doc
}
