/* -*- Mode: C; indent-tabs-mode: nil; c-basic-offset: 4 -*- */
/* vim: set sw=4 sts=4 expandtab: */
/*
   rsvg-paint-server.h : RSVG colors

   Copyright (C) 2000 Eazel, Inc.
   Copyright (C) 2002 Dom Lachowicz <cinamod@hotmail.com>

   This program is free software; you can redistribute it and/or
   modify it under the terms of the GNU Library General Public License as
   published by the Free Software Foundation; either version 2 of the
   License, or (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
   Library General Public License for more details.

   You should have received a copy of the GNU Library General Public
   License along with this program; if not, write to the
   Free Software Foundation, Inc., 59 Temple Place - Suite 330,
   Boston, MA 02111-1307, USA.

   Author: Raph Levien <raph@artofcode.com>
*/

#ifndef RSVG_PAINT_SERVER_H
#define RSVG_PAINT_SERVER_H

#include <glib.h>
#include <cairo.h>

G_BEGIN_DECLS 

typedef struct _RsvgSolidColor RsvgSolidColor;

typedef struct _RsvgPaintServer RsvgPaintServer;

/* Implemented in rust/src/gradient.rs */
G_GNUC_INTERNAL
RsvgNode *rsvg_node_linear_gradient_new (const char *element_name, RsvgNode *parent);

/* Implemented in rust/src/gradient.rs */
G_GNUC_INTERNAL
RsvgNode *rsvg_node_radial_gradient_new (const char *element_name, RsvgNode *parent);

/* Implemented in rust/src/gradient.rs */
G_GNUC_INTERNAL
gboolean gradient_resolve_fallbacks_and_set_pattern (RsvgNode       *node,
                                                     RsvgDrawingCtx *draw_ctx,
                                                     guint8          opacity,
                                                     RsvgBbox        bbox);
/* Implemented in rust/src/pattern.rs */
G_GNUC_INTERNAL
RsvgNode *rsvg_node_pattern_new (const char *element_name, RsvgNode *parent);

/* Implemented in rust/src/pattern.rs */
G_GNUC_INTERNAL
gboolean pattern_resolve_fallbacks_and_set_pattern (RsvgNode       *node,
                                                    RsvgDrawingCtx *draw_ctx,
                                                    RsvgBbox        bbox);

struct _RsvgSolidColor {
    gboolean currentcolor;
    guint32 argb;
};

typedef struct _RsvgSolidColor RsvgPaintServerColor;
typedef enum _RsvgPaintServerType RsvgPaintServerType;
typedef union _RsvgPaintServerCore RsvgPaintServerCore;
typedef struct _RsvgPaintServerIri RsvgPaintServerIri;

struct _RsvgPaintServerIri {
    char *iri_str;
    gboolean has_alternate;
    RsvgSolidColor alternate;
};

union _RsvgPaintServerCore {
    RsvgSolidColor *color;
    RsvgPaintServerIri *iri;
};

enum _RsvgPaintServerType {
    RSVG_PAINT_SERVER_SOLID,
    RSVG_PAINT_SERVER_IRI
};

struct _RsvgPaintServer {
    int refcnt;
    RsvgPaintServerType type;
    RsvgPaintServerCore core;
};

/* Create a new paint server based on a specification string. */
G_GNUC_INTERNAL
RsvgPaintServer	    *rsvg_paint_server_parse    (gboolean *inherit, const char *str);
G_GNUC_INTERNAL
void                 rsvg_paint_server_ref      (RsvgPaintServer * ps);
G_GNUC_INTERNAL
void                 rsvg_paint_server_unref    (RsvgPaintServer * ps);

G_GNUC_INTERNAL
RsvgNode *rsvg_new_linear_gradient  (const char *element_name, RsvgNode *parent);
G_GNUC_INTERNAL
RsvgNode *rsvg_new_radial_gradient  (const char *element_name, RsvgNode *parent);

/* Implemented in rust/src/stop.rs */
G_GNUC_INTERNAL
RsvgNode *rsvg_node_stop_new (const char *element_name, RsvgNode *parent);


G_END_DECLS

#endif
