# MathJax TeX Packages

## base

Inline: $ (a+b)^3 = a^3 + 3a^2b + 3ab^2 + b^3 $

$$
\left(\sum_{k=1}^{n} k\right)^2
= \left(\frac{n(n+1)}{2}\right)^2
$$

---

## ams

$$
\begin{align}
\operatorname{Var}(X)
&= \mathbb{E}\!\left[(X-\mathbb{E}[X])^2\right] \\
&= \mathbb{E}[X^2] - \left(\mathbb{E}[X]\right)^2
\end{align}
$$

$$
\begin{equation}
\begin{split}
\int_0^1 x^p(1-x)^q\,dx
&= \frac{\Gamma(p+1)\Gamma(q+1)}{\Gamma(p+q+2)} \\
&\eqqcolon \mathrm{B}(p+1,q+1)
\end{split}
\end{equation}
$$

---

## amscd

$$
\begin{CD}
0 @>>> A @>>> B @>>> C @>>> 0 \\
@. @VV\alpha V @VV\beta V @VV\gamma V @. \\
0 @>>> A' @>>> B' @>>> C' @>>> 0
\end{CD}
$$

---

## boldsymbol

$$
\boldsymbol{\nabla} \times \boldsymbol{E}
= -\frac{\partial \boldsymbol{B}}{\partial t}
$$

---

## braket

$$
\langle \psi | H | \phi \rangle
= \sum_n \langle \psi | n \rangle \langle n | \phi \rangle
$$

---

## bussproofs

$$
\begin{prooftree}
\AxiomC{$A \rightarrow B$}
\AxiomC{$B \rightarrow C$}
\BinaryInfC{$A \rightarrow C$}
\RightLabel{\scriptsize (Trans)}
\AxiomC{$A$}
\BinaryInfC{$C$}
\end{prooftree}
$$

---

## cancel

$$
\frac{(x-1)\cancel{(x+1)}}{\cancel{(x+1)}} = x-1
\qquad
\bcancel{a+b}
\qquad
\xcancel{0}
$$

---

## cases

$$
f(x)=
\begin{cases}
\displaystyle \int_0^x t^2\,dt & \text{if } x \ge 0 \\
\displaystyle -\sum_{k=1}^{\infty}\frac{x^{2k-1}}{2k-1} & \text{if } x < 0
\end{cases}
$$

---

## centernot

$$
A \centernot\subseteq B
\qquad
x \centernot\equiv y \pmod{n}
$$

---

## color

$$
{\color{red}{\int_0^\infty e^{-x}\,dx}} = {\color{blue}{1}}
\qquad
{\color{green}{(a+b)^2}} = a^2 + 2ab + b^2
$$

---

## empheq

$$
\begin{empheq}[left=\empheqlbrace]{align}
y &= mx + b \\
m &= \frac{y_2-y_1}{x_2-x_1}
\end{empheq}
$$

---

## enclose

$$
\enclose{circle}{x}
\quad
\enclose{box}{x+1}
\quad
\enclose{roundedbox}{\frac{a}{b}}
$$

---

## extpfeil

$$
A \xRightarrow[\text{lower label}]{\text{upper label}} B
\qquad
X \xleftrightarrow{\text{iso}} Y
$$

---

## gensymb

$$
45\degree,\quad 298\,\mathrm{K} \approx 25\celsius,\quad 1\,\mathrm{\ohm}
$$

---

## mathtools

$$
\DeclarePairedDelimiter{\normM}{\lVert}{\rVert}
\normM{x}_2 \coloneqq \sqrt{\sum_{i=1}^n x_i^2}
\qquad
a \eqqcolon b
$$

---

## mhchem

$$
\ce{2H2(g) + O2(g) -> 2H2O(l)}
$$

$$
\ce{Fe^{2+} ->[\text{oxidation}] Fe^{3+} + e-}
$$

---

## noundefined

$$
E = mc^2
\quad
\ThisMacroIsNotDefined
\quad
\int_0^1 x\,dx = \frac12
$$

---

## upgreek

$$
\alpha,\ \beta,\ \gamma
\qquad
\upalpha,\ \upbeta,\ \upgamma
\qquad
\Delta,\ \updelta
$$

---

## unicode

$$
∀x∈\mathbb{R},\ \exists y≥0:\ y^2 = x^2
$$

---

## verb

$$
\verb|git commit -m "mathjax test"|
\qquad
\verb|\alpha + \beta|
$$

---

## tagformat

$$
\begin{equation}
\label{eq:tagtest}
\int_0^\infty e^{-x}\,dx = 1
\tag{TAG-INT}
\end{equation}
(\ref{eq:tagtest})
$$

---

## textcomp

$$
\text{\textnumero 123 \texttrademark\ \textregistered}
$$

---

## textmacros

$$
\textbf{Bold},\ \textit{Italic},\ \texttt{Monospace},
\text{and }
x=\frac{-b\pm\sqrt{b^2-4ac}}{2a}
$$

---

## physics

$$
\dv{x}{t} = v,\qquad \dv[2]{x}{t} = a
$$

$$
\grad \phi,\ \div \vec{E},\ \curl \vec{B}
$$

$$
\qty(\frac{\sin x}{x})^2
\qquad
\comm{A}{B}=AB-BA
$$

---

## newcommand

$$
\newcommand{\R}{\mathbb{R}}
\newcommand{\inner}[2]{\langle #1,#2\rangle}
\newcommand{\normv}[1]{\lVert #1\rVert}
$$

$$
x\in\R,\quad
\inner{x}{y}=x^\mathsf{T}y,\quad
\normv{x}^2=\inner{x}{x}
$$

---

## bbox

$$
\bbox[5px,border:2px solid red]{x^2 + y^2}
\qquad
\bbox[background:yellow]{\int_0^\infty e^{-x}\,dx}
$$

---

## require (NOT loaded: negative test)

The following should **fail or be ignored** if the `require` package is not loaded:

$$
\require{physics}
$$

$$
\require{cancel}
$$

---

## action (NOT loaded: negative test)

The following should **fail or render as plain content** if the `action` package is not loaded:

$$
\action{toggle}{x^2 + y^2}
$$

$$
\action{tooltip}{E=mc^2}
$$

---

## Integrated Stress Test

$$
\bbox[8px,border:1px solid grey]{
\begin{aligned}
\text{\color{blue}{State}}:\ \ket{\psi}
&= \sum_{n\in\mathbb{N}} c_n \ket{n} \\
\text{\color{red}{Energy}}:\
E &= \mel{\psi}{H}{\psi}
\qquad
\dv{E}{t} \centernot= 0 \\
\text{\color{green}{Chem}}:\
&\ce{2H2 + O2 -> 2H2O}
\end{aligned}
}
$$
