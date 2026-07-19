# ============================================================

# ⚙️ RemapH V3

# 03_Runtime.md

# ============================================================

## 🎯 Propósito

Este documento define la arquitectura actual del Runtime de RemapH V3.

El Runtime recibe entradas físicas genéricas y resuelve remapeos compilados.

No conoce la UI.

No conoce Windows.

No conoce Interception.

---

# 📑 Índice

1. Objetivo
2. Responsabilidad
3. Flujo general
4. Modelo persistente
5. Compilador
6. Perfil compilado
7. Cache
8. Motor de ejecución
9. Estados
10. Reglas de diseño

---

# 🎯 1. Objetivo

El Runtime transforma un `InputEvent` en una decisión de ejecución.

Su flujo principal es:

```
InputEvent
↓
Buscar coincidencia
↓
Resultado
```

El resultado puede ser:

```
Pasar
Esperar
Consumir
```

Si existe una acción compilada, el Runtime la envía a la salida.

---

# 🏛️ 2. Responsabilidad

El Runtime contiene:

• Compilación.
• Modelo compilado.
• Cache.
• Resolución de Triggers.
• Estado lógico de Inputs.
• Resolución de acciones.

El Runtime no contiene:

• Botones.
• UI.
• JSON.
• Windows API.
• Interception.
• Captura física.

---

# 🔄 3. Flujo general

El flujo de configuración es:

```
UI
↓
PerfilJson
↓
Compilador
↓
PerfilCache
↓
Cache
```

El flujo de ejecución es:

```
Entrada física
↓
InputEvent
↓
Runtime
↓
Cache
↓
AccionCache
↓
Salida física
```

---

# 📄 4. Modelo persistente

La configuración persistente se representa mediante `PerfilJson`.

El modelo contiene la información necesaria para reconstruir la configuración del usuario.

El Runtime no lee directamente este modelo.

---

# 🔨 5. Compilador

`compilador.rs` convierte:

```
PerfilJson
↓
PerfilCache
↓
Cache
```

Actualmente el compilador:

• Ignora remapeos con estado `OFF`.
• Requiere un gatillo válido.
• Requiere una acción válida.
• Convierte entradas a `InputId`.
• Construye `RemapeoCache`.

El compilador no ejecuta acciones.

---

# 📦 6. Perfil compilado

`perfilcache.rs` define las estructuras internas del Runtime.

```
RemapeoCache
│
├── TriggerCache
└── AccionCache
```

`TriggerCache` contiene:

```
modificadores[]
gatillo
```

`AccionCache` representa la acción preparada para ejecución.

Este modelo:

• No se serializa.
• No conoce JSON.
• No conoce UI.

---

# 💾 7. Cache

`cache.rs` almacena los `RemapeoCache` compilados.

La Cache:

• Reemplaza el conjunto completo.
• Busca Triggers exactos.
• Busca Pulses.
• Detecta prefijos.

La Cache no contiene información visual.

---

# ⚙️ 8. Motor de ejecución

`runtime.rs` contiene `Estado`.

El Runtime mantiene:

• Orden de Inputs activos.
• Inputs consumidos.

El orden de Inputs es importante porque la Cache compara los modificadores en orden.

---

## Resultado

El Runtime devuelve:

### Pasar

El evento no corresponde a un remapeo consumido.

La entrada puede continuar.

### Esperar

La entrada actual puede ser el inicio de un Trigger incompleto.

El Runtime espera más Inputs.

### Consumir

El remapeo coincidió.

El evento se considera consumido.

---

# 🟢 9. Estados

El Runtime consulta el estado global del perfil.

Si el perfil está inactivo:

```
InputEvent
↓
Pasar
```

El estado global vive en `estado.rs`.

El Runtime no decide cómo cambiarlo.

---

# 📌 10. Reglas de diseño

## El Runtime no lee JSON

Toda configuración debe pasar por compilación.

## El Runtime no conoce la UI

Puede ejecutarse sin conocer los controles visuales.

## El Runtime no conoce el hardware

Trabaja con `InputEvent` e `InputId`.

## El Runtime no interpreta nombres humanos

La conversión ocurre antes de llegar al Runtime.

## La Cache contiene modelos compilados

No debe contener información visual ni persistente.

## Runtime decide

La Platform ejecuta físicamente.

---

# ✅ Resumen

El Runtime es el motor lógico de RemapH.

Su flujo actual es:

```
InputEvent
↓
Runtime
↓
Cache
↓
AccionCache
```

Su responsabilidad es resolver remapeos compilados.

No conoce la UI.

No conoce Windows.

No conoce la configuración persistente.
