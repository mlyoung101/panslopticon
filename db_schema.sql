--
-- PostgreSQL database dump
--

\restrict G8hO6fyCtRsueDo1fUiAtrvCUbHbUgx1lwQDT9RiVEo96KaUFbduwYHgnnZYSPD

-- Dumped from database version 18.6
-- Dumped by pg_dump version 18.6

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET transaction_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: _sqlx_migrations; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public._sqlx_migrations (
    version bigint NOT NULL,
    description text NOT NULL,
    installed_on timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL,
    success boolean NOT NULL,
    checksum text NOT NULL,
    execution_time bigint NOT NULL
);


ALTER TABLE public._sqlx_migrations OWNER TO postgres;

--
-- Name: agents; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.agents (
    slop_id bigint NOT NULL,
    agent text NOT NULL
);


ALTER TABLE public.agents OWNER TO postgres;

--
-- Name: full_text; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.full_text (
    slop_id bigint NOT NULL,
    file text NOT NULL,
    text text NOT NULL
);


ALTER TABLE public.full_text OWNER TO postgres;

--
-- Name: gh_metrics; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.gh_metrics (
    slop_id bigint NOT NULL,
    date text NOT NULL,
    stars bigint NOT NULL,
    forks bigint NOT NULL
);


ALTER TABLE public.gh_metrics OWNER TO postgres;

--
-- Name: gh_metrics_slop_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.gh_metrics_slop_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.gh_metrics_slop_id_seq OWNER TO postgres;

--
-- Name: gh_metrics_slop_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.gh_metrics_slop_id_seq OWNED BY public.gh_metrics.slop_id;


--
-- Name: ham; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.ham (
    id bigint NOT NULL,
    url text NOT NULL,
    date_added timestamp without time zone NOT NULL,
    score real NOT NULL,
    panslop_version text,
    origin_platform text,
    origin_src text,
    dead bigint DEFAULT 0 NOT NULL,
    date_last_seen timestamp without time zone NOT NULL
);


ALTER TABLE public.ham OWNER TO postgres;

--
-- Name: ham_full_text; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.ham_full_text (
    id bigint NOT NULL,
    file text NOT NULL,
    text text NOT NULL
);


ALTER TABLE public.ham_full_text OWNER TO postgres;

--
-- Name: ham_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.ham_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.ham_id_seq OWNER TO postgres;

--
-- Name: ham_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.ham_id_seq OWNED BY public.ham.id;


--
-- Name: ingress; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.ingress (
    id bigint NOT NULL,
    url text NOT NULL,
    date_added timestamp without time zone NOT NULL,
    origin_platform text NOT NULL,
    origin_src text NOT NULL
);


ALTER TABLE public.ingress OWNER TO postgres;

--
-- Name: ingress_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.ingress_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.ingress_id_seq OWNER TO postgres;

--
-- Name: ingress_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.ingress_id_seq OWNED BY public.ingress.id;


--
-- Name: not_slop; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.not_slop (
    id bigint NOT NULL,
    url text NOT NULL,
    date_added timestamp without time zone NOT NULL,
    score real NOT NULL
);


ALTER TABLE public.not_slop OWNER TO postgres;

--
-- Name: not_slop_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.not_slop_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.not_slop_id_seq OWNER TO postgres;

--
-- Name: not_slop_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.not_slop_id_seq OWNED BY public.not_slop.id;


--
-- Name: slop; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.slop (
    id bigint NOT NULL,
    url text NOT NULL,
    date_added timestamp without time zone NOT NULL,
    score real NOT NULL,
    panslop_version text NOT NULL,
    date_last_seen timestamp without time zone NOT NULL,
    dataset_path text,
    origin_platform text NOT NULL,
    origin_src text NOT NULL,
    dead bigint DEFAULT 0 NOT NULL
);


ALTER TABLE public.slop OWNER TO postgres;

--
-- Name: slop_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.slop_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.slop_id_seq OWNER TO postgres;

--
-- Name: slop_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.slop_id_seq OWNED BY public.slop.id;


--
-- Name: gh_metrics slop_id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.gh_metrics ALTER COLUMN slop_id SET DEFAULT nextval('public.gh_metrics_slop_id_seq'::regclass);


--
-- Name: ham id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.ham ALTER COLUMN id SET DEFAULT nextval('public.ham_id_seq'::regclass);


--
-- Name: ingress id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.ingress ALTER COLUMN id SET DEFAULT nextval('public.ingress_id_seq'::regclass);


--
-- Name: not_slop id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.not_slop ALTER COLUMN id SET DEFAULT nextval('public.not_slop_id_seq'::regclass);


--
-- Name: slop id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.slop ALTER COLUMN id SET DEFAULT nextval('public.slop_id_seq'::regclass);


--
-- Name: _sqlx_migrations idx_261022_PRIMARY; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public._sqlx_migrations
    ADD CONSTRAINT "idx_261022_PRIMARY" PRIMARY KEY (version);


--
-- Name: gh_metrics idx_261049_PRIMARY; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.gh_metrics
    ADD CONSTRAINT "idx_261049_PRIMARY" PRIMARY KEY (slop_id);


--
-- Name: ham idx_261060_PRIMARY; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.ham
    ADD CONSTRAINT "idx_261060_PRIMARY" PRIMARY KEY (id);


--
-- Name: ingress idx_261082_PRIMARY; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.ingress
    ADD CONSTRAINT "idx_261082_PRIMARY" PRIMARY KEY (id);


--
-- Name: not_slop idx_261094_PRIMARY; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.not_slop
    ADD CONSTRAINT "idx_261094_PRIMARY" PRIMARY KEY (id);


--
-- Name: slop idx_261105_PRIMARY; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.slop
    ADD CONSTRAINT "idx_261105_PRIMARY" PRIMARY KEY (id);


--
-- Name: idx_261022_sqlite_autoindex__sqlx_migrations_1; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX idx_261022_sqlite_autoindex__sqlx_migrations_1 ON public._sqlx_migrations USING btree (version);


--
-- Name: idx_261082_sqlite_autoindex_ingress_1; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX idx_261082_sqlite_autoindex_ingress_1 ON public.ingress USING btree (id);


--
-- Name: idx_261082_sqlite_autoindex_ingress_2; Type: INDEX; Schema: public; Owner: postgres
--

CREATE UNIQUE INDEX idx_261082_sqlite_autoindex_ingress_2 ON public.ingress USING btree (url);


--
-- Name: idx_261094_not_slop_url_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_261094_not_slop_url_idx ON public.not_slop USING btree (url);


--
-- Name: idx_261105_slop_url_idx; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_261105_slop_url_idx ON public.slop USING btree (url);


--
-- Name: full_text fk_full_text_slop_id; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.full_text
    ADD CONSTRAINT fk_full_text_slop_id FOREIGN KEY (slop_id) REFERENCES public.slop(id);


--
-- Name: gh_metrics fk_gh_metrics_slop_id; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.gh_metrics
    ADD CONSTRAINT fk_gh_metrics_slop_id FOREIGN KEY (slop_id) REFERENCES public.slop(id);


--
-- Name: ham_full_text fk_ham_full_text_id; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.ham_full_text
    ADD CONSTRAINT fk_ham_full_text_id FOREIGN KEY (id) REFERENCES public.ham(id);


--
-- PostgreSQL database dump complete
--

\unrestrict G8hO6fyCtRsueDo1fUiAtrvCUbHbUgx1lwQDT9RiVEo96KaUFbduwYHgnnZYSPD
