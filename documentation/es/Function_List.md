# Mégra Referencia de Funciones

La tabla de contenido está agrupada por categorías, la lista a continuación está ordenada alfabéticamente.

Tabla de Contenido
==================

**Generadores**:

Cree generadores de secuencia de eventos básicos.

* [cyc - Cycle Generator](#cyc---cycle-generator)
* [chop - Chop a sample](#chop---chop-a-sample)
* [friendship - Create Friendship (or Windmill) Generator](#friendship---create-friendship-generator)
* [flower - Create Flower Generator](#flower---create-flower-generator)
* [fully - Create Fully Connected Generator](#fully---create-fully-connected-generator)
* [infer - Infer Generator from Rules](#infer---infer-generator-from-rules)
* [learn - Learn Generator from Distribution](#learn---learn-generator-from-distribution)
* [lin - Simple Linear Sequence](#lin---simple-linear-sequence)
* [loop - Simple Loop Generator](#loop---simple-loop-generator)
* [nuc - Nucleus Generator](#nuc---nucleus-generator)
* [stages - Arrange an Event Sequence in Stages](#stages---stages-generator)

**Modificadores de Generadores**:

Los modificadores de generadores modifican la estructura, los pesos o el orden / velocidad de evaluación de los generadores básicos.

* [blur - Blur Probabilities](#blur---blur-probabilities)
* [discourage - Stir Up Generator](#discourage---stir-up-generator)
* [encourage - Consolidate Generator](#encourage---consolidate-generator)
* [grow - Enlarge Generator](#grow---enlarge-generator)
* [haste - speed up evaluation](#haste---speed-up-evaluation)
* [keep - Persistence](#keep---persistent-generator)
* [life - Manipulate Generator](#life---manipulate-generator)
* [relax - Slow Down Generator](#relax---slow-down-generator)
* [rewind - Rewind Generator](#rewind---rewind-generator)
* [sharpen - Sharpen Probabilities](#blur---sharpen-probabilities)
* [shrink - Shrink Generator](#shrink---shrink-generator)
* [skip - Skip Events](#skip---skip-events)
* [solidify - Solidify Generator](#solidify---solidify-generator)

**Multiplicadores de Generadores**:

Los multiplicadores de generadores duplican los generadores básicos y eventualmente aplican modificadores (modificadores de flujo de eventos o generador).

* [xdup - Multiply Generators with Modifiers](#xdup---multiply-generators-with-modifiers)
* [xspread - Multiply Generators with Modifiers, Spread over Loudspeakers](#xspread---multiply-generators-with-modifiers-and-spread-over-channels)
* [ls - Create Generator List](#ls---create-generator-list)

**Modificadores de Parámetros**:

Parámetros dinámicos.

* [brownian - Bounded Brownian Motion](#brownian---bounded-brownian-motion)   
* [env - Parameter Envelope](#env---parameter-envelope)
* [exh - Event Stream Manipulator](#exh---event-stream-manipulator)
* [fade - Parameter Fader](#fade---parameter-fader)
* [inh - Event Stream Manipulator](#inh---event-stream-manipulator)
* [bounce - Parameter Oscillator](#bounce---parameter-oscillator)

**Applicadores**:

Aplique modificaciones a los generadores o al flujo de eventos que los atraviesa. Basado en conteo o probabilidad.

* [pear - Apply Event Modifiers](#pear---apply-event-modifiers)
* [apple - Probability-Based Generator Manipulators](#apple---probability-based-generator-manipulators)
* [every - Count-Based Generator Manipulators](#every---count-based-generator-manipulators)

**Misc**:

Ayudantes, gestión de sesiones, etc.

* [cmp - Compose Generators](#cmp---compose-generators)
* [clear - Clear Session](#clear---clear-session)     
* [control - Control Functions](#ctrl---control-functions)
* [sx - Event Sinks](#sx---multiple-event-sinks)
* [export-dot - Export to DOT File](#export-dot---export-to-dot-file)
* [defpart - Define Parts](#defpart---define-parts)
* [step-part - Evaluate Parts Step by Step](#step-part---evaluate-parts-step-by-step)
* [rec - Record](#rec---record-session)
* [stop-rec - stop recording](#rec---record-session)

Lista de Funciones Alfabéticas
=============================

## `apple` - Probability-Based Generator Manipulators

Modificar un generador con cierta probabilidad

### Ejemplo

```lisp
(sx 'ba #t 
    (apple :p 10 (skip 2) :p 9 (rewind 2) ;; skip with probability of 10%, rewind with chance of 9%
        (cyc 'ba "bd ~ hats ~ sn ~ hats ~")))
```

## `blur` - Blur Probabilities

Distribuye los pesos de manera más uniforme, de modo que la secuencia generada se vuelve menos predecible.

### Syntax

`(blur <blur factor>)`

### Parámetros

* `blur factor` - Cantidad de desenfoque, donde 0.0 no tiene ningún efecto y 1.0 desenfoca mucho.

### Ejemplo

```lisp
;; inferir un ciclo con repeticiones ocasionales
(sx 'con #t 
  (infer 'duct :events 
    'a (saw 'a2)
    'b (saw 'f2)
    'c (saw 'c3)
    :rules 
      (rule 'a 'a 10 200) 
      (rule 'a 'b 90 200) 
      (rule 'b 'b 10 400) 
      (rule 'b 'c 90 400) 
      (rule 'c 'c 10 100) 
      (rule 'c 'a 90 100)))
```

![A sharp loop.](./diagrams/sharp.svg) 

```lisp
;; now blur
(sx 'con #t 
  (blur 0.8 
    (infer 'duct :events 
      'a (saw 'a2)
      'b (saw 'f2)
      'c (saw 'c3)
      :rules 
          (rule 'a 'a 10 200) 
          (rule 'a 'b 90 200) 
          (rule 'b 'b 10 400) 
          (rule 'b 'c 90 400) 
          (rule 'c 'c 10 100) 
          (rule 'c 'a 90 100))))
```

![A less sharp loop.](./diagrams/blurred.svg) 

```lisp
;; you can use blur over time, too
(sx 'con #t 
  (every :n 10 (blur 0.1) ;; blur every 10 steps
    (infer 'duct :events 
      'a (saw 'a2)
      'b (saw 'f2)
      'c (saw 'c3)
      :rules 
          (rule 'a 'a 10 200) 
          (rule 'a 'b 90 200) 
          (rule 'b 'b 10 400) 
          (rule 'b 'c 90 400) 
          (rule 'c 'c 10 100) 
          (rule 'c 'a 90 100))))
```

## `brownian` - Bounded Brownian Motion 

Defina un movimiento browniano acotado en un parámetro.

### Parámetros

* lower boundary (float)
* upper boundary (float)
* `:wrap` (boolean) (t) - wrap value if it reaches lower or upper boundary
* `:step` (float) (0.1) - step that the parameter will be incremented/decremented

### Syntax

```lisp
(brownian <lower boundary> <upper boundary> :wrap <wrap> :step <step-size>)
```

### Ejemplos
	
```lisp
(sx 'some #t
    (cmp (pear (rate (brownian 0.8 1.2)))
         (nuc 'bass (saw 120 :dur 200))))
```

## `chop` - Chop a sample

Corte un *sample* en partes, que se reproducirán como un bucle. Todos los demás parámetros de
se puede aplicar un bucle (`rep`,` max-rep` y `rnd`).

### Ejemplos

```lisp
;; chop violin sample into 8 parts (each of which is 200ms long)
(sx 'some #t
  (chop 'chops 8 (violin 'a3 :sus 200))) 
```

## `clear` - Cerrar 

Detiene y elimina todos los generadores presentes.

### Ejemplos

```lisp
(sx 'some #t
  (cyc 'bear "bd ~ hats ~ sn ~ hats ~"))

(sx 'more #t :sync 'some
  (cyc 'bass "saw:100 ~"))

(clear) ;; cerrar todo
```

## `cmp` - Compose Generators

### Syntax
```lisp
(cmp <generators>)
```

### Ejemplos

```lisp
;; The plain lisp approach would be:
(sx 'composed #t
    (pear (rev 0.1)
		(every :n 20 (haste 2 0.5)
			(cyc 'bl "bd ~ ~ sn ~ ~"))) )

;; this makes it somewhat inconvenient to add/disable certain parts.

;; With cmp, it can be re-written as:
(sx 'composed #t
    (cmp
		(pear (rev 0.1))
		(every :n 20 (haste 2 0.5))
		(cyc 'bl "bd ~ ~ sn ~ ~")
		))

;; now individual modifiers can easily be commented out

```

## `ctrl` - Control Functions

Ejecuta cualquier función, se puede utilizar para realizar la ejecución de generadores.

### Parámetros

* function

### Syntax

```lisp
(ctrl <function>)
```

### Ejemplo

```lisp
;; define some parts
(defpart 'bass 	
		(nuc 'bass (saw 100)))
	
(defpart 'mid 
	(nuc 'midrange (saw 1000)))
	
(defpart 'treble 
	(nuc 'treble (saw 5000)))

;; Define a score, here as a learned one, even 
;; though any other generator might be used.
(sx 'ga #t 
	(learn 'ta 
		:events
		'a (ctrl (sx 'ba #t 'bass))
		'b (ctrl (sx 'ba #t 'mid))
		'c (ctrl (sx 'ba #t 'treble))
		:sample "ababaabbababcabaababaacabcabac"
		:dur 2400))
```

## `cyc` - Cycle Generator

Genera un ciclo (también conocido como 'loop') a partir de un lenguaje de secuenciación simple. Puede especificar parámetros dentro del lenguaje de secuencia,
o marcadores de posición. Además, puede especificar desviaciones de la duración predeterminada entre eventos dentro del lenguaje de secuenciación.
Parece simple, pero este es uno de los generadores más potentes de Mégra.

### Parámetros

* name - nombre del generador
* sequence - sequence description
* `:dur` - default space between events 
* `:rep` - probability of repeating an event
* `:max-rep` - limits number of repetitions
* `:rnd` - random connection probability (currently not working the way I expected it ...)
* `:map` - map events on parameters
* `:events` - use labeled events

### Syntax

```lisp
(cyc <name> :dur <duration> :rep <repetition probability> :max-rep <max number of repetitions> :rnd <random connection prob> <sequence>)
```

### Ejemplo 
```lisp
;; plain
(sx 'simple #t
  (cyc 'beat "bd ~ hats ~ sn ~ hats ~"))
```

![A plain beat](./diagrams/cycle-simple.svg) 

```lisp
;; with a 40% chance of repetition, 2 times at max
(sx 'simple #t
    (cyc 'beat :rep 40 :max-rep 2 "bd ~ hats ~ sn ~ hats ~"))
```
![A beat with repetitions](./diagrams/cycle-complex.svg)


```lisp
;; with labeled events
(sx 'simple #t	
	(cyc 'beat 
	:events 'a (bd) 'b (hats) 'c (sn)
	"'a ~ 'b ~ 'c ~ 'b ~"))
```

```lisp
;; with parameters and placeholder
(sx 'simple #t	
	(cyc 'beat 
	:map 'saw 
	"200 ~ 120 140 'a3")) ;; you can use frequencies or note names 
```

```lisp
;; with escape durations
(sx 'simple #t
	(cyc 'beat "bd ~ hats /100 hats /100 ~ sn ~ hats ~"))
```

```lisp
;; ciclos de control con otros ciclos
(sx 'control #t
	(cyc 'ba 
		:dur 1599 ;; switch just in time ... will run out of sync eventually
		:events
		'a (ctrl (sx 'controlled #t (cyc 'fa "bd sn")))
		'b (ctrl (sx 'controlled #t (cyc 'fa "hats hats")))
		"'a 'b 'a 'b"
		))
```

## `defpart` - Define Parts

Defina partes, que son básicamente listas de generadores que puede nombrar.
Las partes no se actualizan en los contextos de sincronización en ejecución, es decir, cuando
modificar la parte, también debe volver a evaluar el contexto de sincronización.

### Syntax
`(defpart <part-id> <generator-list>)`

### Ejemplo

```lisp
(defpart 'drum-and-bass 
  (cyc 'drums "bd ~ sn ~")
  (nuc 'bass (saw 100)))
	
(defpart 'hats-and-mid 
  (cyc 'hats "hats hats ~ hats")
  (nuc 'midrange (saw 1000)))
	
(defpart 'treble-and-cym
  (cyc 'cym "cym ~ cym cym")
  (nuc 'treble (saw 2000)))

;; Define a score, here as a learned one, even 
;; though any other generator might be used.
(sx 'ga #t 
  (learn 'ta 
    :events 
    'a (ctrl (sx 'ba #t 'drum-and-bass))
    'b (ctrl (sx 'ba #t 'hats-and-mid))
    'c (ctrl (sx 'ba #t 'treble-and-cym))
    :sample "ababaabbababcabaababaacabcabac"
    :dur 2400))
```

## `env` - Parameter Envelope

Defina una envolvente en cualquier parámetro. La longitud de la lista de niveles debe ser uno más que la longitud de la lista de duraciones.
Las duraciones se basan en pasos, por lo que las duraciones absolutas dependen de la velocidad a la que funciona el generador.

### Parámetros

* `:v` or `:values` - level points on envelope path
* `:s` or `:steps` - transition durations (in steps)
* `:repeat` (boolean) - loop envelope 

### Syntax

```lisp
(env :values/:v <levels> :steps/:s <durations> :repeat <#t/#f>)
```

### Ejemplo

```lisp
(sx 'simple #t
    (cmp 
        (pear (lvl (env :v 0.0 0.4 0.0 :s 20 30)))
        (cyc 'beat "bd ~ hats ~ sn ~ hats ~")))
```

## `every` - Count-Based Generator- and Event Stream Manipulator

Cada uno de los pasos, haz algo con el generador o el flujo de eventos.

### Ejemplos

```lisp
(sx 'simple #t
    (cmp 
	    (every :n 20 (skip 2) :n 33 (rewind 2)) ;; <- every 20 steps, skip 2, every 33, rewind 2
        (cyc 'beat "bd ~ hats ~ sn ~ hats ~")))
```

## `exh` - Event Stream Manipulator

Exhibir tipo de evento, es decir, silenciar todos los demás eventos, con cierta probabilidad.

### Parámetros

* probablility (int) - exhibit probablility
* filter (filter function) - event type filter

### Syntax
```lisp
(exh <probability> <filter>)
```

### Ejemplo
```lisp
(sx 'simple #t 
  (cmp 
      (exh 30 'hats)
      (exh 30 'bd)
      (nuc 'beat (bd) (sn) (hats)))) 
```

## `export-dot` - Export to DOT File

Export a generator (or at least its underlying structure) to a DOT file that can be 
rendered with GraphViz.

## Syntax
`(export-dot <filename> <generator> or <keyword> and <tag list>)`

### Ejemplo
```
;; if a generator is provided, it will be exported as a DOT file directly
(export-dot "dotdotdot.dot"
	(cyc 'bu "saw ~ saw ~ saw ~"))
	
;; if the keyword "live" and a tag list are provided, all running generators matching the tag list will be exported
(sx 'ba #t 
  (cyc 'bu "bd ~ sn ~"))
  
(export-dot "babu" :live 'ba 'bu)

;; if a part is defined ... 
(defpart 'ga
	(cyc 'du "cym cym cym cym")
	(cyc 'ba "bd ~ sn ~"))
	
;; you can export it using the "part" keyword
(export-dot "partpart" :part 'ga)

```

## `fade` - Parameter Fader

Fade a parameter (sinusoidal).

### Syntax

`(fade <from> <to> :steps <steps>)`

### Ejemplo
```lisp

;; fade cutoff frequency
(sx 'osc #t
    (nuc 'ill (saw 300 :lp-freq (fade 300 2200 :steps 20))))

;; same, but a different position 
(sx 'osc #t
    (cmp
     (pear (lpf (fade 300 2200 :steps 4)))
     (nuc 'ill (saw 300))))

;; fade duration
(sx 'osc #t
    (nuc 'ill (saw 300) :dur (fade 100 400)))

;; fade probablility
(sx 'osc #tt
    (cmp
     (pear :p (fade 0 100) (lvl 0.0))
     (nuc 'ill (saw 300))))
```

## `friendship` - Create Friendship Generator

Esto crea una versión dirigida de un grafo tipo *amistad* o *molina de viento*.

### Syntax

`(friendship <name> :center <center event> :friends <list of events>)`

### Parámetros

* `name` - nombre
* `:center` - el centro del "circulo social"
* `:friends` - l@s "amig@s"
* `:rep` - chance of repetition.
* `:max-rep` - maximum number of repetitions
* `:rnd` - generate random shortcuts
* `:events` - collect labeled events

### Ejemplo

```lisp
(sx 'friend #t
  (cmp
    (pear (atk 1) (rel 90) (sus 10) (rev 0.07))
      (friendship 'ship 
        :dur 100
		:center  (saw 'a2) 
        :friends (saw 'c3) (saw 'e3) (saw 'b3) (saw 'd3) (saw 'f3) (saw 'c4))))
```

![A friendly bassline](./diagrams/friendship.svg)

## `flower` - Create Flower Generator

Crear... bueno, mira los ejemplos.

### Syntax:
`(flower <name> :pistil <event> :layers <layers> :petals <events>)`

### Parámetros:

* `name` - nombre
* `:layers` - numero de capas
* `:pistil` - pistilo o evento central
* `:petals` - list of events (will be padded to appropriate lenght if necessary)

### Ejemplos

```lisp
;; flower with one layer and four petals
(sx 'a-rose-is-a #t
  (flower 'rose 
    :pistil (saw 100)
    :petals (saw 200) (saw 300) (saw 400) (saw 150)))
```

![Flower with one layer and four petals.](./diagrams/flower-one-layer.svg)

Flower with 2 layers:
```lisp
(sx 'a-rose-is-a #t
  (flower 'rose 
    :layers 2
    :pistil (saw 100)
    :petals (saw 200) (saw 300) (saw 400) (saw 150) 
            (saw 400) (saw 600) (saw 800) (saw 300)))
```
![Flower with one layer and four petals.](./diagrams/flower-two-layers.svg)

## `fully` - Create Fully Connected Generator

Cada nodo sigue a otro nodo con igual probabilidad... así que básicamente es un generador aleatorio.

### Syntax
```lisp
(fully <name> :rest <list of events> :events <labeled events>)
```

### Ejemplo

```lisp
;; random generator with five events
(sx 'full #t
    (fully 'mel :rest (saw 'a3) (saw 'f2) (saw 'c3) (saw 'e3) (saw 'a4)))

```

![Fully connected graph.](./diagrams/fully-connected.svg)    

## `grow` - Enlarge Generator

El algoritmo de crecimiento permite agregar información a un generador ya existente.
Lo hace eligiendo un evento que el generador produjo en el pasado, sacudiendo los valores
un poco, y agregándolo al generador siguiendo ciertos principios.

### Parámetros

* `:var` (float) - variation factor (smaller -> less variation)
* `:method` (symbol) - growth method/mode (see below)
* `:durs` (list of ints) - durations to mix in
* `:rnd` (int) - chance to add random edges after growth

### Ejemplos

```lisp
(sx 'al #t
	(every :n 10 (grow) 
		(nuc 'gae (sqr 120))))
```

### Modes

Each growth mode pushes the generator in a certain direction.

* `'default`
* `'triloop`
* `'quadloop`
* `'flower`
* `'loop` 

## `haste` - Speed Up Evaluation

Speed up evaluation for a specified number of steps, by a certain ratio.

### Ejemplos

```lisp
(sx 'more #t
    (xspread     
     (cmp ;; this is another copy with modifiers ...
          (pear (freq-mul 3.0))
          (every :n 20 (haste 4 0.5))) ;; <- every 20 steps, double-time for four steps. Change to 0.25 for quadruple-time, etc
     ;; this is the "original" 
     (cyc 'one "tri:120 tri:90 tri:100 tri:120 ~ ~ ~ ~")))
```

## `inh` - Event Stream Manipulator

Inhibit event type, that is, mute event of that type, with a certain probability.

### Parámetros

* probablility (int) - inhibit probablility
* filter (filter function) - event type filter

### Syntax

```lisp
(inh <probability> <filter>)
```

### Ejemplo

```lisp
(sx 'simple #t
  (cmp (inh 30 'hats)
       (inh 30 'bd)
       (inh 30 'sn)
       (nuc 'beat (bd) (sn) (hats))))
```

## `life` - Manipulate Generator

This is one of the more complex generator manipulations. To understand it, it's helpful to play around 
with the `(grow ...)` function first. What the `(life ...)` method does is basically the same, but automated
and bound to resources, in the fashion of a primitive life-modeling algorithm:

* There's a global pool of resources (which is just an abstract number).
* Each generator is assigned an amount of local resources (same as above).
* Each time the generator grows, it comes at a cost, which is subtracted first from the local, then from the global resources.
* If all resources are used up, nothing can grow any further.

Each symbol in the current alphabet is assigned an age (the number of times it has been evaluated), so at a certain, specified
age they can perish, freeing a certain amount of resources (which are added to the local resource pool).

Furthermore, if specified, the generator can be configured to "eat itself up" when a shortage of resources occurs. That means 
that an element will be removed before its time, freeing resources for further growth (which, again, are added to the local resources).

### Parámetros

* growth cycle (int)
* average lifespan (int)
* variation (float)
* `:method` - growth method (see `(grow ...)`)
* `:durs` - list of possible durations to choose from
* `:apoptosis` (bool) - if `nil`, symbols never die
* `:autophagia` (bool) - if `t`, the generator will eat its own symbols to generate energy for further growth

```
;; define global resources
(global-resources 30000)
```

### Ejemplos

The algorithm is quite configurable, but to use the default configuration, you can simply use:

```lisp
(sx 'the-circle #t
    (life 10 12 0.4 ;; first arg: growth cycle, second arg: average lifespan, third arg: variation factor
          (cyc 'of-life "tri:100 tri:120 ~ ~ tri:120 tri:180 ~ tri:200")))
```

This means that every ten steps the generator will grow, while the average lifespan of an element is 12 evaluations.
The a variantion factor of 0.4 will be applied when generating the new elements.

You can specify a growth method (see the paragraph on `(grow ...)` for details):

```lisp
(sx 'the-circle #t
    (life 10 12 0.4 :method 'flower
          (cyc 'of-life "tri:100 tri:120 ~ ~ tri:120 tri:180 ~ tri:200")))
```

To add some rhythmical variation, you can mix in other durations (chosen at random):

```lisp
(sx 'the-circle #t
    (life 10 12 0.4 :method 'flower :durs 100 200 200 200 400 100
          (cyc 'of-life "tri:100 tri:120 ~ ~ tri:120 tri:180 ~ tri:200")))
```

Another interesting way to use this is to juxtapose it with a static generator (note the reset flag is nil'd so
we can change parameters without starting from scratch every time):

```lisp
(sx 'the-circle #t
    (xspread
		(pear (freq-mul 1.5) (life 10 12 0.4 :method 'flower :durs 100 200 200 200 200 400))
		(cyc 'of-life "tri:100 tri:120 ~ ~ tri:120 tri:180 ~ tri:200")))
```

## `ls` - Create Generator List

Si desea modificar varios generadores, puede recopilarlos en una lista.

### Ejemplo

```lisp
(sx 'ba #t
  (pear (rev 0.1) 
    (ls ;; thanks to ls, you can apply the reverb to everything at once 
      (cyc 'drum "bd ~ sn ~")
      (cyc 'bass "saw:'a1 ~ ~ ~"))))
```

## `nuc` - Generador Nucleo

Genera un generador repetitivo de un nodo, es decir, como punto de partida para el crecimiento.

### Parámetros

* nombre (symbol)
* event(s) (event or list of events) - events to be repeated
* `:dur` - transition duration between events

### Syntax

```lisp
(nuc <name> :dur <duration> <event(s)>)
```

### Ejemplo

```lisp
;; with one event
(sx 'just #t
  (nuc 'a-bassdrum :dur 400 (bd)))
  
;; with multiple events
(sx 'just #t
  (nuc 'a-bassdrum-and-a-snare :dur 400 (bd) (sn)))
```
![Just a Bassdrum](./diagrams/nucleus.svg)

## `bounce` - Parameter Oscillator

Defina la oscilación en cualquier parámetro. La curva de oscilación es un poco hinchable, no realmente sinusoidal.

### Parámetros 

* upper limit - límite superior de oscilación
* lower limit - límite inferior de oscilación
* `:cycle` - duración del ciclo de oscilación en pasos

### Syntax

```lisp
(bounce <upper limit> <lower limit> :cycle <cycle length in steps>)
```

### Ejemplo

```lisp
(sx 'simple #t
  (nuc 'beat (bd) :dur (bounce 200 600 :steps 80)))
```

## `infer` - Infer Generator from Rules

Inferir un generador a partir de reglas arbitrarias. Asegúrese de que cada evento tenga
al menos una salida, de lo contrario el generador se detendrá.

Además, las probabilidades de salida para cada nodo deben sumar 100.

### Parámetros

* `name` - nombre del generador
* `:events` - mapeo de eventos etiquetados
* `:rules` - reglas de transición - Formato `(rule <source> <destination> <probability> <duration (optional)>)`

### Ejemplo

```lisp
;; infer 
(sx 'con #t 
  (infer 'duct :events 
    'a (saw 'a2)
    'b (saw 'f2)
    'c (saw 'c3)
    'd (saw 'e4)
    :rules 
    (rule 'a 'a 80 200) ;; repeat 'a with 80% chance
    (rule 'a 'b 20 200) ;; move to 'b with 20% chance
    (rule 'aaa 'c 100 200) ;; after 3 repetitions of 'a, always move to 'c
    (rule 'b 'b 100 400) ;; repeat 'b always
    (rule 'bb 'd 100 400) ;; ... well, 2x max
    (rule 'c 'c 100 100) ;; same for 'c
    (rule 'ccc 'a 100 400) 
    (rule 'd 'd 80 200) ;; 'd is repeated with 80% chance as well
    (rule 'd 'a 20 200) ;; and moves back to 'a with 20% chance
    (rule 'ddddd 'b 100 400))) ;; and is repeated 5x max

```
![An inferred bassline.](./diagrams/inferred-generator.svg)

## `learn` - Learn Generator from Distribution

Learn a generator from a sample string. Based on the variable-order Markov chain learning algorithm
proposed in *Ron, Singer, Tishby - The Power of Amnesia (1996)*.

### Parámetros
* `:events` - Event definitions.
* `:sample` - Sample string to learn from. Uses the defined event mapping as characters.
* `:bound` - The maximum order of the learned markov chain, that is, how far to look back when determining the next step.
* `:epsilon` - Probability threshold, a connection that's less likely than that won't be learned. The higher, the longer it takes to learn.
* `:size` - Maximum generator size (nodes in the probabilistic finite automaton generated).
* `:autosilence` - Use `~` as default character for silence.

### Ejemplo
Learn a trap-like beat from a sample string.
```lisp
(sx 'from #t
  (learn 'data
    :events 'x (bd) 'o (sn) 'h (hats)
    :sample "xoxoxoxox~~o~h~~~h~h~h~~h~h~~hhh~x~o
             ~x~o~x~o~x~o~xh~h~~hhh~x~o~x~o~x~o~x
             ox~xox~xox~xoxo~xoxo~xoxox~oooo~xxxx
             ~xoxoxox~ohxhohxhohxhxhxhxhxhxhxhoho
             hoh"))
```
<img src="./diagrams/learned-beat.svg" alt="A learned beat." width="1000" height="1000">

## `lin` - Simple Linear Sequence

Si solo necesita una secuencia lineal simple (sin repetición), este es el camino a seguir. Esto es
excelente para escribir partituras, usando la secuencia lineal con eventos de control para anotar otros generadores.

### Example

```lisp
;; default durations
(sx 'conductor #t
  (lin 'score 
    (ctrl (sx 'part #t (cyc 'ga "bd ~ sn ~"))) 4000 ;; si no proporcionas un duracion, se usa la duracion defecta
	(ctrl (sx 'part #t (cyc 'ga "bd hats sn hats"))) 4000
	(ctrl (sx 'part #t (cyc 'ga "[bd cym] cym [sn cym] cym"))) 4000
	(ctrl (clear))
	))

```

## `loop` - Simple Loop Generator

El generador `cyc` es un animal compleja, más o menos un lenguaje peque por sí mismo. El generador de bucles es un
generador muy simple si desea un *loop* simple en una sintaxis lisp-y.

### Example

```lisp
;; duraciones predeterminadas
(sx 'around #t
  (loop 'and-around (saw 100) (saw 200) (saw 300) (saw 400)))
  
;; duraciones individuales
(sx 'around #t
  (loop 'and-around (saw 100) 400 (saw 200) 100 (saw 300) 200 (saw 400)))
```

## `pear` - Apply Event Modifiers

Appl-ys and Pears (don't ask me why it's named like this, I like good pears and found it funny).

### Ejemplo

```lisp
(sx 'ba #t
    (cmp
        (pear (freq-mul 1.5)) ;; <- always multiply frequency by 1.5
		(pear :p 10 (freq-mul 0.5) :p 20 (freq-mul 2.0)) ;; <- with a probablility of 10%, multiply with 0.5, and 20% multiply by two
		(pear :p 20 (rev 0.2)) ;; <- with a probability of 20%, apply reverb
		(pear :for 'sqr :p 20 (freq-mul 1.7)) ;; <- only for sqr events, apply a multiplicator with a chance of 20%
	    (cyc 'ta "saw:150 ~ sqr:100 ~ saw:150 ~ sqr:100")))
```

## `relax` - Slow Down Generator

Slows down generator for a specified number of steps, by a certain ratio.

### Ejemplos

```lisp
(sx 'more #t
    (xspread     
     (cmp ;; this is another copy with modifiers ...
          (pear (freq-mul 3.0))
          (every :n 20 (relax 4 0.5))) ;; <- every 20 steps, half-time for four steps
     ;; this is the "original" 
     (cyc 'one "tri:120 tri:90 tri:100 tri:80 ~ ~ tri:120 tri:90 tri:100 tri:80 ~")))
```

## `rewind` - Rewind Generator

Re-winds the generator by a specified number of steps. The further unfolding might
be different from the previous one, obviously.

### Ejemplos

```lisp
(sx 'more #t
    (xspread     
     (cmp ;; this is another copy with modifiers ...
          (pear (freq-mul 3.0))
          (every :n 20 (rewind 2))) ;; <- every 20 steps, rewind 2 steps (only the copy)
     ;; this is the "original" 
	 (cyc 'one "tri:120 tri:90 tri:100 tri:80 ~ ~ tri:120 tri:90 tri:100 tri:80 ~")))
```

## `shrink` - Shrink Generator
Removes a symbol from the generator's alphabet. While `grow` adds symbols based on 
the existing ones, this will remove them.

### Ejemplo
```lisp
;; this will grow faster than it shrinks
(sx 'ba #t
    (every :n 10 (grow 0.2) :n 20 (shrink) 
        (nuc 'ba (saw 120))))
```

## `skip` - Skip Events
Skips ahead a specified number of steps.

### Ejemplo

```lisp
(sx 'more #t
    (xspread     
     (cmp ;; this is another copy with modifiers ...
          (pear (freq-mul 3.0))
          (every :n 20 (skip 2))) ;; <- every 20 steps, skip 2 steps ahead (only the copy)
     ;; this is the "original" 
	 (cyc 'one "tri:120 tri:90 tri:100 tri:80 ~ ~ tri:120 tri:90 tri:100 tri:80 ~")))
```

## `solidify` - Solidify Generator

Mira el historial de símbolos emitidos de un generador y agrega una conexión de orden superior para hacer
la última secuencia es más probable que vuelva a suceder.

### Ejemplo

```lisp
(defpart 'ga
  (every 
    :n 8 (solidify 4) ;; <- four last symbol emissions
    (infer 'ta :events 'a (sqr 550) 'b (sqr 200) 'c (sqr 300) 'd (sqr 400)
      :rules
      (rule 'a 'a 10 100)
      (rule 'a 'b 90 100)
      (rule 'b 'a 80 100)
      (rule 'b 'c 20 100)
      (rule 'c 'd 100 100)
      (rule 'd 'a 100 100) 
      )))

;; step 8 times
(step-part 'ga)
```
Generado inicial:

![before_solidification](./diagrams/before_solidification.svg)

Después de la solidificación:

![after_solidification](./diagrams/after_solidification.svg)

## `stages` - Stages Generator

Este generador organiza los eventos de sonido en "etapas". Ver por ti mismo.

### Syntax

`(stages <name> :pprev <prob> :pnext <prob> :dur <duration> <events>)`

### Parámetros

* `name` - nombre del generator
* `:dur` - duración entre eventos
* `:pprev` - probabilidad de adelantar al proximo evento
* `:pnext` - probabilidad de volver al evento previo
* `:cyc` - cíclico (la última etapa avanzará a la primera etapa)

### Ejemplo
```lisp
;; non-cyclical
(sx 'ba #t
  (stages 'ga :pprev 10 :pnext 10 (saw 100) (saw 200) (saw 300) (saw 400)))
```
![stages generator](./diagrams/stages-non-cyclical.svg)

```lisp
;; cyclical
(sx 'ba #t
  (stages 'ga :pprev 10 :pnext 10 (saw 100) (saw 200) (saw 300) (saw 400)))
```
![cyclical stages generator](./diagrams/stages-cyclical.svg)

## `step-part` - Evaluate Parts Step by Step

Define a part and evaluate it step by step. This exists mostly for debugging purposes.

### Ejemplo

```lisp
(defpart 'ba ;; <-- define some part
  (cyc 'hu "hats cym hats cym cym hats hats cym")
  (cyc 'du "bd ~ sn ~ bd bd sn ~"))

(step-part 'ba) ;; <-- step through ...
```

## `sx` - Event Sink

Corto para `SyncconteXt`.

### Ejemplo

```lisp
(sx 'simple #t
  (nuc 'beat (bd) :dur 400))

(sx 'simple2 #t :sync 'simple :shift 200
  (nuc 'beat2 (sn) :dur 400))
  
;; you can solo and mute by tag ...
  
(sx 'solo #t :solo 'bd ;; <-- solo all events tagged 'bd'
  (nuc 'snare (sn) :dur 400)
  (nuc 'hats (hats) :dur 400)
  (nuc 'bass (bd) :dur 400))

(sx 'solo #t :block 'bd ;; <-- block all events tagged 'bd'
  (nuc 'snare (sn) :dur 400)
  (nuc 'hats (hats) :dur 400)
  (nuc 'bass (bd) :dur 400))
```

## `xdup` - Multiply Generators with Modifiers

Si desea yuxtaponer (referencia obvia aquí) un generador con una copia modificada de sí mismo,
sin volver a escribir todo el generador.

### Ejemplo
```lisp
(sx 'more #t
    (xdup
     (cmp ;; this is the copy with modifiers ...
      (pear (freq-mul 2.0) (rev 0.1))
      (every :n 20 (haste 2 0.5)))     
     ;; this is the "original" 
     (cyc 'one "tri:'f3 tri:'a3 tri:'c4 tri:'e4 ~ ~ tri:'f3 tri:'a3 tri:'c4 tri:'e4 ~")))
```

## `xspread` - Multiply Generators with Modifiers and spread over Channels.

Si desea yuxtaponer (referencia obvia aquí) un generador con una copia modificada de sí mismo,
sin volver a escribir todo el generador. A diferencia de `xdup`, este distribuye las copias entre
los altavoces/canales disponibles, o el espectro espacial (una vez que esté disponible el estéreo binaural o los ambisónicos).

### Ejemplo
```lisp
(sx 'more #t
    (xspread
     (cmp ;; this is the copy with modifiers ...
          (pear (freq-mul 2.0) (rev 0.1))
          (every :n 20 (haste 2 0.5)))
     (cmp ;; this is another copy with modifiers ...
          (pear (freq-mul 4.02) (rev 0.1))
          (every :n 20 (haste 3 0.5)))     
     ;; this is the "original" 
     (cyc 'one "tri:'f3 tri:'a3 tri:'c4 tri:'e4 ~ ~ tri:'f3 tri:'a3 tri:'c4 tri:'e4 ~")))
```



