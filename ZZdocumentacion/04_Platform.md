# ============================================================

# 🪟 RemapH V3

# 04_Platform.md

# ============================================================

## 🎯 Propósito

Este documento define la arquitectura actual de Platform.

Platform es la frontera entre RemapH y Windows.

Su responsabilidad es:

```
Hardware / Windows
↓
Platform
↓
InputEvent
```

y:

```
AccionCache
↓
Platform
↓
Hardware / Windows
```

Platform no decide qué remapeo debe ejecutarse.

---

# 📑 Índice

1. Objetivo
2. Responsabilidad
3. Modos de entrada
4. Entrada Full
5. Entrada Portable
6. Salida física
7. Tauri
8. Comunicación con Runtime
9. Reglas de diseño

---

# 🎯 1. Objetivo

Platform permite que RemapH interactúe con Windows sin exponer detalles físicos al Runtime.

La entrada física se convierte en:

`InputEvent`

La salida recibe acciones compiladas.

---

# 🏛️ 2. Responsabilidad

Platform contiene:

• Interception.
• Windows API.
• Hooks de teclado.
• Hooks de mouse.
• SendInput.
• Captura física.
• Salida física.

Platform no contiene:

• Reglas de remapeo.
• Perfil UI.
• Perfil JSON.
• Cache.
• Decisiones de coincidencia.

---

# 🔀 3. Modos de entrada

`entrada.rs` define dos modos:

```
Full
Portable
```

Ambos utilizan el mismo Runtime.

La diferencia está en el backend físico.

---

# 🟦 4. Entrada Full

El modo Full utiliza Interception.

Flujo:

```
Interception
↓
back_interception
↓
InputEvent
↓
Runtime
```

`back_interception.rs`:

• Inicializa Interception.
• Configura filtros.
• Recibe Strokes.
• Traduce Strokes.
• Reenvía eventos originales.

No conoce el Runtime.

---

# 🟨 5. Entrada Portable

El modo Portable utiliza Windows API.

Utiliza:

• WH_KEYBOARD_LL.
• WH_MOUSE_LL.
• SendInput.

Flujo:

```
Windows Hook
↓
back_windows
↓
InputEvent
↓
Runtime
```

El backend Portable no conoce:

• Cache.
• Remapeos.
• Runtime.

Solo traduce eventos físicos y emite eventos genéricos.

---

# ▶️ 6. Salida física

La salida depende del modo físico.

## Full

`back_salida.rs` utiliza Interception.

Flujo:

```
AccionCache
↓
back_salida
↓
Interception
```

## Portable

`back_windows.rs` utiliza SendInput.

Flujo:

```
AccionCache
↓
entrada.rs
↓
back_windows
↓
SendInput
```

La acción sigue siendo genérica antes de llegar al backend físico.

---

# 🖥️ 7. Tauri

Tauri es el puente entre la UI y la lógica nativa.

`comandos.rs` expone comandos para:

• Compilar perfil.
• Activar perfil.
• Desactivar perfil.
• Iniciar captura.
• Obtener captura.
• Obtener perfil actual.

El flujo de configuración es:

```
UI
↓
Tauri
↓
PerfilJson
↓
Persistencia / Compilador
```

Tauri no decide coincidencias de remapeos.

---

# 🔗 8. Comunicación con Runtime

La comunicación mantiene una separación simple:

```
Platform recibe InputEvent
↓
Runtime decide
↓
Platform emite AccionCache
```

Platform no interpreta el Trigger.

Platform no busca remapeos.

Platform no decide si un evento debe consumirse.

---

# 📌 9. Reglas de diseño

## Platform no decide

Platform ejecuta traducciones físicas.

## Platform no conoce perfiles

No debe saber qué es un perfil de usuario.

## Platform no conoce la Cache

La Cache pertenece al Runtime.

## Platform traduce

Convierte formatos físicos en formatos internos.

## Runtime decide, Platform ejecuta

Esta separación debe mantenerse.

---

# ✅ Resumen

Platform es la frontera física de RemapH.

Actualmente soporta dos caminos de entrada:

```
Full
↓
Interception
```

y:

```
Portable
↓
Windows API
```

Ambos entregan `InputEvent` al mismo Runtime.
