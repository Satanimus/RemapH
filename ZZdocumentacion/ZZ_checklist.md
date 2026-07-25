PLAN — Nuevo sistema de análisis de triggers RemapH V3
Objetivo final

Pasar de:

InputEvent
↓
"¿es modificador conocido?"
↓
"si no, es gatillo"
↓
EventoTrigger

a:

InputEvent
↓
AnalizadorTrigger
↓
Lista temporal de posibilidades
↓
Consulta Cache
↓
Decisión: - esperar - liberar - ejecutar remapeo
ETAPA 1 — Redefinir la responsabilidad del AnalizadorTrigger
Objetivo

Eliminar la idea de:

"algunas teclas son modificadores"

y reemplazarla por:

"cualquier input puede participar en un trigger".

Pasos generales
1.1 Eliminar dependencia de modificadores predefinidos

Eliminar progresivamente:

cache::es_modificador()

como criterio de análisis.

La cache será la única fuente de verdad.

1.2 Cambiar estado interno del analizador

Actualmente:

AnalizadorTrigger
{
modificadores_activos
}

Cambiar hacia:

AnalizadorTrigger
{
eventos_pendientes
}

Guardando:

InputId
Estado
Instante
1.3 Mantener orden físico

El analizador debe respetar:

primer Down
segundo Down
tercer Down
...
último Down = gatillo

Ejemplo:

Ctrl Down
A Down
Mouse Down

produce:

Modificadores:
Ctrl
A

Gatillo:
Mouse
1.4 Validar arquitectura

Al terminar esta etapa:

Debe existir una base donde cualquier dispositivo pueda ser:

modificador
gatillo
acción
ETAPA 2 — Crear motor de candidatos usando Cache
Objetivo

Que el analizador pueda preguntar:

"¿Esto todavía puede convertirse en algún trigger?"

Pasos generales
2.1 Extender Cache

Actualmente tenemos:

buscar()
tiene_prefijo()

Evolucionar hacia consultas más generales:

Ejemplo:

Entrada:

[
Ctrl
]

Respuesta:

Puede continuar:
Ctrl+A
Ctrl+B
Ctrl+Mouse
2.2 Crear concepto de candidato

El analizador debe poder recibir:

Ctrl

y saber:

esperar

No porque Ctrl sea modificador.

Sino porque:

existe un trigger que comienza así.
2.3 Resolver tres estados posibles

El analizador debe devolver internamente:

ESPERAR

LIBERAR

COINCIDENCIA

Ejemplo:

Caso 1
A

Cache:

A doble

Resultado:

ESPERAR
Caso 2
X

Cache:

Ctrl+A

Resultado:

LIBERAR X
Caso 3
Ctrl+A

Cache:

Ctrl+A

Resultado:

COINCIDENCIA
ETAPA 3 — Implementar condición Simple / Doble / Mantenido
Objetivo

Que el analizador pueda diferenciar:

A
A x2
A mantenido
Pasos generales
3.1 Simple

Regla:

A Down
esperar tiempoDoble

Si no ocurre:

A = Simple
3.2 Doble

Ejemplo:

A Down
A Up
A Down

Dentro de:

CONFIG_CAPTURA.tiempoDoble

Resultado:

A = Doble
3.3 Mantenido

Ejemplo:

A Down
...
300ms
...
A sigue presionado

Resultado:

A = Mantenido

Usando:

tiempoMantenido
ETAPA 4 — Rediseñar EventoTrigger
Objetivo

Que represente correctamente el resultado del análisis.

Actualmente:

EventoTrigger
{
modificadores
gatillo
condición
}

Está bien como concepto.

Pero debemos decidir si necesita agregar:

orden original

o

secuencia

para soportar:

Ctrl+A+Click

y futuras variantes.

Pasos:

4.1 Revisar estructura

Confirmar que soporta:

Mouse como modificador
Joystick como modificador
Teclado como gatillo
4.2 Mantener Runtime limpio

Runtime sigue sin saber:

tiempos
buffers
doble tap
mantenido
ETAPA 5 — Adaptar Runtime
Objetivo

Que Runtime trabaje con el nuevo resultado.

Pasos generales
5.1 Mantener filosofía

Runtime:

NO:

analiza
espera
decide

Solo:

EventoTrigger
↓
buscar cache
↓
ejecutar acción
5.2 Implementar consumo correcto

Cuando hay coincidencia:

Debe:

Consumir todos los inputs involucrados

Ejemplo:

Ctrl+A

No debe pasar:

Ctrl
A

a Windows.

ETAPA 6 — Capturador UI usando el mismo analizador
Objetivo

Que captura y runtime tengan exactamente la misma lógica.

Ejemplo:

Usuario captura:

Ctrl+A+Click mantenido

Debe guardar:

Modificadores:
Ctrl
A

Gatillo:
Click

Condición:
Mantenido

Pasos:

6.1 Eliminar lógica paralela del capturador

No crear:

Analizador captura
Analizador runtime

Debe existir:

Un único sistema de interpretación.
6.2 Conectar captura con cache temporal

Durante captura:

Puede usar:

perfil actual
remapeos existentes
modo captura especial
ETAPA 7 — Configuración editable de tiempos
Objetivo

Que exista:

En UI:

Tiempo Doble: 250ms
Tiempo Mantenido: 300ms

Pasos:

7.1 Mantener fuente única

No duplicar:

frontend 250
backend 250
7.2 Guardar configuración

Flujo:

UI
↓
Configuración
↓
Persistencia
↓
Analizador
ETAPA 8 — Nuevo sistema "+" de restricciones
Objetivo

Preparar futuras condiciones.

Primera versión:

Popup:

- |
  └── Win +

Arquitectura futura:

Trigger
|
+-- Inputs
|
+-- Restricciones
|
+-- Monitor
+-- Posición mouse
+-- Programa
+-- Estado dispositivo
ETAPA 9 — Limpieza final
Objetivo

Eliminar restos de arquitectura vieja.

Revisión:

nombres internos
comentarios antiguos
funciones obsoletas
tests
documentación
Orden recomendado de ejecución

La secuencia correcta sería:

1. AnalizadorTrigger nuevo
   ↓
2. Cache preparada para candidatos
   ↓
3. Condiciones Simple/Doble/Mantenido
   ↓
4. EventoTrigger definitivo
   ↓
5. Runtime adaptado
   ↓
6. Captura usando mismo sistema
   ↓
7. Configuración editable
   ↓
8. Restricciones "+"
   ↓
9. Limpieza
   Punto de control importante

Cuando terminemos la Etapa 5, RemapH ya debería ser capaz de:

✅ Ctrl+A
✅ Ctrl+A mantenido
✅ A doble
✅ Mouse+Mouse
✅ Joystick+Teclado
✅ Orden de pulsación real
✅ Sin depender de "teclas modificadoras conocidas"

Ese será el primer estado realmente sólido de la arquitectura nueva.

//////////////////////////////////////////////
📌Solución BUG hook teclado: Agregar esta linea en src-tauri\src\lib.rs
.device_event_filter(tauri::DeviceEventFilter::Always)
////////////////////////////////////////////

📌 Ideas pendientes (no hacer todavía)
Arquitectura
⬜ Evaluar si AnalizadorTrigger debe convertirse únicamente en el analizador lógico mientras BufferEventos decide cuándo una secuencia está completa.
⬜ Elaborar un diccionario oficial de términos del proyecto para evitar ambigüedades futuras.
Rendimiento
⬜ Revisar el tamaño óptimo del BufferEventos.
⬜ Optimizar la reutilización de memoria del buffer.
Extensiones futuras
⬜ Integrar joystick utilizando la misma arquitectura de captura.
⬜ Evaluar soporte para nuevos tipos de triggers si fueran necesarios.

------ Recordar limpiar comandos.rs
------ igual captura.rs (eliminar)

------Documentar oficialmente el flujo del motor, igual que hicimos con el flujo Perfil → Compilador → Cache → Runtime.

Algo como:
Windows / Interception
│
▼
CapturadorTrigger
│
▼
BufferEventos
│
▼
AnalizadorTrigger
│
▼
ProcesadorEventos
│
▼
Runtime
│
▼
Emisor

Ese diagrama será muy útil cuando dentro de unos meses tengamos que volver al código.

----- Cuando terminemos el motor, podríamos hacer que BufferEventos tenga un modo de depuración que imprima la línea temporal completa. Algo como:

00.000 Ctrl Down
00.120 A Down
00.145 A Up
00.310 Ctrl Up

Sería una herramienta excelente para depurar problemas de triggers sin tocar el resto del sistema.
