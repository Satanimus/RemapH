# ============================================================

# 📖 RemapH V3

# 06_Principios.md

# ============================================================

## 🎯 Propósito

Este documento reúne las reglas permanentes de arquitectura de RemapH V3.
No describe una implementación concreta.
Describe las decisiones que no deberían cambiar aunque el código evolucione.
Cuando exista una duda de diseño, este documento tiene prioridad sobre cualquier implementación existente.

---

# 📑 Índice

1. Filosofía del proyecto
2. Responsabilidad única
3. Arquitectura por capas
4. Comunicación
5. Componentes
6. Eventos
7. Contexto compartido
8. Escalabilidad
9. Filosofía visual
10. Regla de decisión

---

# 🎯 1. Filosofía del proyecto

RemapH busca resolver problemas complejos mediante una arquitectura simple.
La complejidad debe existir únicamente donde sea estrictamente necesaria.
Siempre se preferirá:

• simplicidad
• claridad
• desacoplamiento
• mantenibilidad

antes que soluciones ingeniosas o difíciles de comprender.

---

# 📌 2. Responsabilidad única

## Regla

Cada archivo debe tener una única responsabilidad.

## Motivo

Cuando un archivo intenta hacer varias cosas, termina creciendo sin control y se vuelve difícil de mantener.
Esta regla aplica igualmente a:

• carpetas
• componentes
• módulos
• documentos

---

# 🏛️ 3. Arquitectura por capas

## Regla

Toda responsabilidad pertenece exactamente a una capa.

```
UI
↓
Core
↓
Runtime
↓
Platform
```

## Motivo

Cada capa resuelve un tipo distinto de problema.
Nunca deben mezclarse responsabilidades entre ellas.

---

# 🔄 4. Comunicación

## Regla

Los componentes no deben conocerse directamente siempre que pueda evitarse.
La comunicación debe realizarse mediante mecanismos comunes.

## Motivo

Eliminar dependencias directas facilita reemplazar componentes sin modificar el resto del sistema.

---

# 🧩 5. Componentes

## Regla

Un componente debe representar una única idea.
Puede contener otros componentes.
No debe asumir responsabilidades ajenas.

## Ejemplos

Una fila representa un remapeo.
Una caja Acción representa únicamente el contenido asociado a una acción.
Un Popup representa únicamente una forma de selección.

---

# 📡 6. Eventos

## Regla

Un evento representa un hecho.
Nunca representa una orden.

## Correcto

```
EVT_TIPO
```

## Incorrecto

```
CambiarAccion
ActualizarBoton
RefrescarFila
```

## Motivo

Los eventos informan.
Los receptores deciden.
Nunca ocurre al revés.

---

## Los datos pertenecen al evento

El nombre del evento debe permanecer simple.
Toda la información necesaria viaja dentro del propio evento.

Ejemplo:

```
Evento
EVT_TIPO
Datos
id
valor
```

El receptor decide si ese evento le corresponde.

---

# 🪪 7. Contexto compartido

## Regla

Cada fila posee un único ContextoFila (ID).
Todos los componentes creados dentro de esa fila reciben exactamente la misma instancia.

## Motivo

La identidad de una fila nunca cambia.
Los componentes pueden destruirse y reconstruirse sin perder esa identidad.

---

# 📈 8. Escalabilidad

## Regla

Siempre que sea posible, una nueva funcionalidad debe agregarse sin modificar componentes existentes.

## Motivo

Una arquitectura estable crece incorporando piezas nuevas, no aumentando el tamaño de las anteriores.

---

# 🎨 9. Filosofía visual

La interfaz debe transmitir una herramienta profesional.
Se evitarán elementos decorativos innecesarios.
La prioridad siempre será:

• legibilidad
• consistencia
• velocidad de uso
• claridad

Antes que apariencia llamativa.

---

# 🧭 10. Regla de decisión

Cuando aparezca una duda durante el desarrollo, deben responderse estas preguntas en orden:

## 1

¿A qué capa pertenece?
Si no puede responderse, la responsabilidad probablemente no está bien definida.

---

## 2

¿Tiene una única responsabilidad?
Si intenta hacer varias cosas, debe dividirse.

---

## 3

¿Necesita conocer otro componente?
Si la respuesta es sí, debe evaluarse si la comunicación puede realizarse mediante Core.

---

## 4

¿Puede crecer sin modificar lo existente?
Si no puede hacerlo, probablemente la arquitectura todavía puede mejorarse.

---

# ✅ Resumen

Antes de escribir código, recordar siempre:

• Una responsabilidad por archivo.
• Una responsabilidad por componente.
• Los eventos representan hechos.
• Los receptores toman decisiones.
• Las capas no deben mezclarse.
• La simplicidad tiene prioridad sobre la complejidad.
Estas reglas constituyen la base arquitectónica permanente de RemapH V3.
