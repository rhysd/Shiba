# GitHub MathJax TeX extension samples

### Basic

inline: $a^2 + b^2 = c^2$

$$
\int_0^\infty e^{-x^2} dx = \frac{\sqrt{\pi}}{2}
$$

---

### AMS（align / cases）

$$
\begin{align}
f(x) &= x^2 + 2x + 1 \\
g(x) &= \frac{1}{x}
\end{align}
$$

$$
\begin{cases}
x + y = 1 \\
x - y = 3
\end{cases}
$$

---

### braket / cancel / color

$$
\langle \psi | H | \psi \rangle
$$

$$
\frac{\cancel{2}x^2}{\cancel{2}x} = x
$$

$$
\color{blue}{x} + \color{red}{y} = z
$$

---

### mhchem

$$
\ce{2H2 + O2 -> 2H2O}
$$

---

### newcommand

$$
\newcommand{\R}{\mathbb{R}}
x \in \R
$$

---

### physics dv, pdv

$$
\dv{x} \sin x = \cos x
$$

$$
\pdv{f}{x} = 2x
$$

---

### physics vb, va

$$
\vb{F} = m \va{a}
$$

---

### physics abs, norm

$$
\abs{x} = \sqrt{x^2}
$$

$$
\norm{\vb{v}} = \sqrt{v_x^2 + v_y^2}
$$
