# ============================================================

# 🧠 RemapH V3

# 02_Core.md

# ============================================================

## 🎯 Propósito

Este documento define la arquitectura interna de Core.
Core es la capa encargada de la comunicación y estructuras compartidas entre componentes.
No contiene lógica de interfaz.
No contiene lógica de remapeo.
No contiene comunicación con hardware.

---

# 📑 Índice

1. Objetivos
2. Responsabilidad de Core
3. Eventos
4. ContextoFila
5. Comunicación entre componentes
6. Flujo de información
7. Reglas de dependencia
8. Buenas prácticas

---

# 🎯 1. Objetivos

Core existe para evitar que los componentes dependan directamente entre ellos.
Su objetivo es permitir que distintas partes del programa puedan comunicarse manteniendo independencia.
La pregunta principal de Core es:
> ¿Cómo se comunican las piezas sin conocerse?

---

# 🏛️ 2. Responsabilidad de Core

Core contiene infraestructura común.
Incluye:
• Eventos.
• Contextos compartidos.
• Sistemas de comunicación.
• Modelos comunes.
• Reglas internas reutilizables.

Core no contiene:
• Botones.
• Ventanas.
• Teclas físicas.
• Reglas de remapeo.
• Acciones del sistema.

---

# 📡 3. Eventos

## Concepto

Un evento representa un hecho ocurrido dentro del sistema.
Un evento no representa una orden.
El componente que emite informa.
El componente que recibe decide qué hacer.

---

## Ejemplo correcto

```
EVT_TIPO
{
 id:"F001",
 valor:"Mouse"
}
```

Significado:
"El tipo de esta fila ahora es Mouse."

---

## Ejemplo incorrecto

```
CAMBIAR_ACCION_A_MOUSE
```

Significado:
"Ejecuta esta instrucción concreta."
Eso acopla componentes.

---

## Nombre del evento

El nombre debe contener únicamente la categoría del hecho.
Ejemplo:

```
EVT_TIPO
EVT_ESTADO
EVT_ACCION
```

La información específica viaja dentro de los datos.

---

# 🪪 4. ContextoFila

Cada fila posee una identidad única.
Esa identidad pertenece a la fila, no a sus botones.
Una fila clon posee una nueva identidad.

```
Fila

ID
│
├── Botón Tipo
├── Botón Estado
├── Botón Acción
└── Botón Color
```

Todos los componentes internos reciben el mismo ContextoFila.

---

## Motivo

Los botones pueden destruirse.
El contenido puede cambiar.
La fila continúa siendo la misma entidad.
Por eso la identidad debe vivir fuera de los componentes temporales.

---

# 🔄 5. Comunicación entre componentes

La comunicación oficial es:

```
Componente
↓
Evento
↓
Componente interesado
```

Ejemplo:
```
Usuario cambia Tipo
↓
Botón Tipo emite EVT_TIPO
↓
Componente Acción recibe EVT_TIPO
↓
Reconstruye su contenido
```

---

## Prohibido

Un componente no debe hacer:

```
Botón Tipo
↓
buscar Botón Acción
↓
modificarlo directamente
```

Porque crea dependencia directa.

---

# 🔁 6. Flujo de información

La información debe viajar siguiendo una dirección clara.

```
Usuario
↓
Componente UI
↓
Core
↓
Evento
↓
Componente UI
```

Core transporta información.
No decide la apariencia.
No decide la acción.

---

# 🔗 7. Reglas de dependencia

Las dependencias permitidas son:

```
UI
↓
Core
↓
Runtime
↓
Platform
```

Las capas superiores pueden utilizar servicios inferiores.
Las capas inferiores nunca conocen las superiores.

---

## Ejemplos

Correcto:

UI utiliza eventos de Core.
Runtime utiliza modelos de Core.


Incorrecto:

Runtime modifica botones.
Core conoce ventanas.

---

# ✅ 8. Buenas prácticas

Antes de crear una comunicación nueva:
Preguntar:
¿Es realmente necesario un evento?
Si la respuesta es directa y pertenece a un único componente, no necesita evento.
Ejemplo:
Un botón que abre su propio menú no necesita evento.

---

Usar eventos cuando:
• varios componentes deben reaccionar.
• el origen no debe conocer los receptores.
• la información representa un hecho ocurrido.

---

Mantener los eventos pequeños.
Preferir:
```
EVT_TIPO
{
id,
valor
}
```

sobre eventos con nombres largos y específicos.

---

# 📌 Resumen

Core es la capa que mantiene unido el proyecto sin crear dependencias.
Sus reglas principales son:
• Los eventos comunican hechos.
• Los receptores deciden.
• La identidad pertenece a la fila.
• Los componentes reciben contexto.
• La comunicación directa entre componentes debe evitarse.
• Core conecta, pero no controla.