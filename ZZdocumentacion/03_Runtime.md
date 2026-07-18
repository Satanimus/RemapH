# ============================================================

# ⚙️ RemapH V3

# 03_Runtime.md

# ============================================================

## 🎯 Propósito

Este documento define la arquitectura del Runtime de RemapH V3.
El Runtime es el motor encargado de ejecutar los remapeos configurados por el usuario.
Recibe eventos internos, consulta la configuración preparada y decide qué acción corresponde ejecutar.
No conoce la interfaz.
No conoce Windows.
No conoce dispositivos físicos.

---

# 📑 Índice

1. Objetivo del Runtime
2. Responsabilidad
3. Flujo general
4. Configuración del usuario
5. Compilador
6. Caché
7. Motor de ejecución
8. Resolución de remapeos
9. Acciones
10. Estados y tiempos
11. Reglas de diseño

---

# 🎯 1. Objetivo del Runtime

El Runtime transforma eventos de entrada en acciones según la configuración activa.
Su función principal es:

```
Evento recibido
↓
Buscar coincidencia
↓
Determinar acción
↓
Solicitar ejecución
```

El Runtime no crea configuraciones.
No edita perfiles.
No muestra información al usuario.

---

# 🏛️ 2. Responsabilidad

El Runtime contiene:
• Motor de ejecución.
• Reglas de remapeo.
• Resolución de coincidencias.
• Gestión de estados activos.
• Interpretación de estructuras compiladas.
• Solicitud de acciones.

---

El Runtime no contiene:
• Botones.
• Ventanas.
• Popups.
• Captura física directa.
• Comunicación visual.
• Código específico de Windows.

---

# 🔄 3. Flujo general

El flujo completo del sistema es:

```
Usuario
↓
UI
↓
Core
↓
Configuración
↓
Compilador
↓
Caché
↓
Runtime
↓
Platform
↓
Hardware
```

Durante la ejecución normal:

```
Entrada física
↓
Platform
↓
Evento interno
↓
Runtime
↓
Acción
↓
Platform
↓
Salida física
```

---

# 📄 4. Configuración del usuario

La configuración editable representa la intención del usuario.
Debe ser cómoda de leer y modificar.
Ejemplo conceptual:

```
Trigger:
Mouse X1

Acción:
Ctrl + C

Condición:
Doble toque

Ámbito:
Photoshop.exe
```

Esta información no está optimizada para ejecución.
Su objetivo es ser comprensible.

---

# 🔨 5. Compilador

El compilador transforma la configuración del usuario en estructuras preparadas para ejecución.
Su función:

```
Configuración editable
↓
Validación
↓
Normalización
↓
Estructura compilada
```

---

El Runtime no debe interpretar configuraciones crudas.
Debe recibir estructuras preparadas.

---

## Motivo

Separar compilación y ejecución permite:
• Mayor velocidad.
• Menos complejidad durante uso real.
• Validación anticipada.
• Menor cantidad de errores.

---

# 💾 6. Caché

La caché almacena la versión compilada de los remapeos.
Su función es evitar reconstruir información constantemente.
Flujo:

```
Usuario modifica configuración
↓
Compilador actualiza caché
↓
Runtime consulta caché
```

---

La caché pertenece al Runtime.
No pertenece a la UI.
No debe contener información visual.

---

# ⚙️ 7. Motor de ejecución

El motor recibe eventos internos y busca coincidencias.
Ejemplo:

```
Evento:

Mouse X1 presionado
↓
Runtime consulta:
¿Existe remapeo?
↓
Sí
↓
¿Cumple condiciones?
↓
Ejecutar acción
```

---

El motor administra:
• Activación.
• Desactivación.
• Condición de pulsación.
• Tiempo de espera.
• Estados internos.

---

# 🎯 8. Resolución de remapeos

Un remapeo se evalúa mediante sus componentes:

```
Trigger
↓
Condicion
↓
Condiciones
↓
Ámbito
↓
Acción
```

---

Ejemplo:

```
Trigger:
Ctrl+A
↓
Condición:
Mantener pulsado
↓
Ámbito:
Global
↓
Acción:
Volumen+
```

---

El Runtime decide si la acción corresponde.
La UI únicamente representa la configuración.

---

# ▶️ 9. Acciones

Las acciones representan operaciones que deben ejecutarse.
Ejemplos:
• Teclado.
• Mouse.
• Multimedia.
• Macro.
• Coordenadas.
• Otras futuras.

---

El Runtime no ejecuta directamente hardware.
Solicita la acción a Platform.

```
Runtime
↓
Solicitud de acción
↓
Platform
↓
Hardware
```

---

# ⏱️ 10. Estados y tiempos

El Runtime administra comportamientos temporales.
Ejemplos:
• Pulsación normal.
• Mantener pulsado.
• Doble toque.
• Retrasos.
• Secuencias.

---

Estos estados pertenecen al motor.
No pertenecen a la UI.
La interfaz únicamente permite configurarlos.

---

# 📌 11. Reglas de diseño

## El Runtime no interpreta configuraciones humanas
Toda configuración debe pasar por compilación.

## El Runtime trabaja con eventos internos
Nunca debe depender de eventos físicos directamente.

## El Runtime no conoce la existencia de la interfaz
Puede ejecutarse sin UI.

## El Runtime no conoce tecnologías externas
Interception, Tauri o Windows pertenecen a Platform.

## El Runtime decide, Platform ejecuta
Esta separación debe mantenerse siempre.

---

# ✅ Resumen

El Runtime es el cerebro de RemapH.
Sus responsabilidades son:
• Recibir eventos internos.
• Consultar configuraciones compiladas.
• Resolver coincidencias.
• Determinar acciones.
• Solicitar ejecuciones.
No sabe quién creó la configuración.
No sabe quién recibe la acción.
Solo ejecuta la lógica del remapeador.