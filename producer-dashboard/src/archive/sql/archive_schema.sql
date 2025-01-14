--
-- PostgreSQL database dump
--

-- Dumped from database version 16.4 (Debian 16.4-1.pgdg120+1)
-- Dumped by pg_dump version 16.4 (Debian 16.4-1.pgdg120+1)

SET statement_timeout = 0;
SET lock_timeout = 0;
SET idle_in_transaction_session_timeout = 0;
SET client_encoding = 'UTF8';
SET standard_conforming_strings = on;
SELECT pg_catalog.set_config('search_path', '', false);
SET check_function_bodies = false;
SET xmloption = content;
SET client_min_messages = warning;
SET row_security = off;

--
-- Name: authorization_kind_type; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.authorization_kind_type AS ENUM (
    'None_given',
    'Signature',
    'Proof'
);


ALTER TYPE public.authorization_kind_type OWNER TO postgres;

--
-- Name: chain_status_type; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.chain_status_type AS ENUM (
    'canonical',
    'orphaned',
    'pending'
);


ALTER TYPE public.chain_status_type OWNER TO postgres;

--
-- Name: internal_command_type; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.internal_command_type AS ENUM (
    'fee_transfer_via_coinbase',
    'fee_transfer',
    'coinbase'
);


ALTER TYPE public.internal_command_type OWNER TO postgres;

--
-- Name: may_use_token; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.may_use_token AS ENUM (
    'No',
    'ParentsOwnToken',
    'InheritFromParent'
);


ALTER TYPE public.may_use_token OWNER TO postgres;

--
-- Name: transaction_status; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.transaction_status AS ENUM (
    'applied',
    'failed'
);


ALTER TYPE public.transaction_status OWNER TO postgres;

--
-- Name: user_command_type; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.user_command_type AS ENUM (
    'payment',
    'delegation'
);


ALTER TYPE public.user_command_type OWNER TO postgres;

--
-- Name: zkapp_auth_required_type; Type: TYPE; Schema: public; Owner: postgres
--

CREATE TYPE public.zkapp_auth_required_type AS ENUM (
    'none',
    'either',
    'proof',
    'signature',
    'both',
    'impossible'
);


ALTER TYPE public.zkapp_auth_required_type OWNER TO postgres;

SET default_tablespace = '';

SET default_table_access_method = heap;

--
-- Name: account_identifiers; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.account_identifiers (
    id integer NOT NULL,
    public_key_id integer NOT NULL,
    token_id integer NOT NULL
);


ALTER TABLE public.account_identifiers OWNER TO postgres;

--
-- Name: account_identifiers_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.account_identifiers_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.account_identifiers_id_seq OWNER TO postgres;

--
-- Name: account_identifiers_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.account_identifiers_id_seq OWNED BY public.account_identifiers.id;


--
-- Name: accounts_accessed; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.accounts_accessed (
    ledger_index integer NOT NULL,
    block_id integer NOT NULL,
    account_identifier_id integer NOT NULL,
    token_symbol_id integer NOT NULL,
    balance text NOT NULL,
    nonce bigint NOT NULL,
    receipt_chain_hash text NOT NULL,
    delegate_id integer,
    voting_for_id integer NOT NULL,
    timing_id integer,
    permissions_id integer NOT NULL,
    zkapp_id integer
);


ALTER TABLE public.accounts_accessed OWNER TO postgres;

--
-- Name: accounts_created; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.accounts_created (
    block_id integer NOT NULL,
    account_identifier_id integer NOT NULL,
    creation_fee text NOT NULL
);


ALTER TABLE public.accounts_created OWNER TO postgres;

--
-- Name: blocks; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.blocks (
    id integer NOT NULL,
    state_hash text NOT NULL,
    parent_id integer,
    parent_hash text NOT NULL,
    creator_id integer NOT NULL,
    block_winner_id integer NOT NULL,
    last_vrf_output text NOT NULL,
    snarked_ledger_hash_id integer NOT NULL,
    staking_epoch_data_id integer NOT NULL,
    next_epoch_data_id integer NOT NULL,
    min_window_density bigint NOT NULL,
    sub_window_densities bigint[] NOT NULL,
    total_currency text NOT NULL,
    ledger_hash text NOT NULL,
    height bigint NOT NULL,
    global_slot_since_hard_fork bigint NOT NULL,
    global_slot_since_genesis bigint NOT NULL,
    protocol_version_id integer NOT NULL,
    proposed_protocol_version_id integer,
    "timestamp" text NOT NULL,
    chain_status public.chain_status_type NOT NULL
);


ALTER TABLE public.blocks OWNER TO postgres;

--
-- Name: blocks_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.blocks_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.blocks_id_seq OWNER TO postgres;

--
-- Name: blocks_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.blocks_id_seq OWNED BY public.blocks.id;


--
-- Name: blocks_internal_commands; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.blocks_internal_commands (
    block_id integer NOT NULL,
    internal_command_id integer NOT NULL,
    sequence_no integer NOT NULL,
    secondary_sequence_no integer NOT NULL,
    status public.transaction_status NOT NULL,
    failure_reason text
);


ALTER TABLE public.blocks_internal_commands OWNER TO postgres;

--
-- Name: blocks_user_commands; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.blocks_user_commands (
    block_id integer NOT NULL,
    user_command_id integer NOT NULL,
    sequence_no integer NOT NULL,
    status public.transaction_status NOT NULL,
    failure_reason text
);


ALTER TABLE public.blocks_user_commands OWNER TO postgres;

--
-- Name: blocks_zkapp_commands; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.blocks_zkapp_commands (
    block_id integer NOT NULL,
    zkapp_command_id integer NOT NULL,
    sequence_no integer NOT NULL,
    status public.transaction_status NOT NULL,
    failure_reasons_ids integer[]
);


ALTER TABLE public.blocks_zkapp_commands OWNER TO postgres;

--
-- Name: epoch_data; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.epoch_data (
    id integer NOT NULL,
    seed text NOT NULL,
    ledger_hash_id integer NOT NULL,
    total_currency text NOT NULL,
    start_checkpoint text NOT NULL,
    lock_checkpoint text NOT NULL,
    epoch_length bigint NOT NULL
);


ALTER TABLE public.epoch_data OWNER TO postgres;

--
-- Name: epoch_data_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.epoch_data_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.epoch_data_id_seq OWNER TO postgres;

--
-- Name: epoch_data_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.epoch_data_id_seq OWNED BY public.epoch_data.id;


--
-- Name: internal_commands; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.internal_commands (
    id integer NOT NULL,
    command_type public.internal_command_type NOT NULL,
    receiver_id integer NOT NULL,
    fee text NOT NULL,
    hash text NOT NULL
);


ALTER TABLE public.internal_commands OWNER TO postgres;

--
-- Name: internal_commands_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.internal_commands_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.internal_commands_id_seq OWNER TO postgres;

--
-- Name: internal_commands_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.internal_commands_id_seq OWNED BY public.internal_commands.id;


--
-- Name: protocol_versions; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.protocol_versions (
    id integer NOT NULL,
    transaction integer NOT NULL,
    network integer NOT NULL,
    patch integer NOT NULL
);


ALTER TABLE public.protocol_versions OWNER TO postgres;

--
-- Name: protocol_versions_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.protocol_versions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.protocol_versions_id_seq OWNER TO postgres;

--
-- Name: protocol_versions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.protocol_versions_id_seq OWNED BY public.protocol_versions.id;


--
-- Name: public_keys; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.public_keys (
    id integer NOT NULL,
    value text NOT NULL
);


ALTER TABLE public.public_keys OWNER TO postgres;

--
-- Name: public_keys_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.public_keys_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.public_keys_id_seq OWNER TO postgres;

--
-- Name: public_keys_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.public_keys_id_seq OWNED BY public.public_keys.id;


--
-- Name: snarked_ledger_hashes; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.snarked_ledger_hashes (
    id integer NOT NULL,
    value text NOT NULL
);


ALTER TABLE public.snarked_ledger_hashes OWNER TO postgres;

--
-- Name: snarked_ledger_hashes_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.snarked_ledger_hashes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.snarked_ledger_hashes_id_seq OWNER TO postgres;

--
-- Name: snarked_ledger_hashes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.snarked_ledger_hashes_id_seq OWNED BY public.snarked_ledger_hashes.id;


--
-- Name: timing_info; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.timing_info (
    id integer NOT NULL,
    account_identifier_id integer NOT NULL,
    initial_minimum_balance text NOT NULL,
    cliff_time bigint NOT NULL,
    cliff_amount text NOT NULL,
    vesting_period bigint NOT NULL,
    vesting_increment text NOT NULL
);


ALTER TABLE public.timing_info OWNER TO postgres;

--
-- Name: timing_info_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.timing_info_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.timing_info_id_seq OWNER TO postgres;

--
-- Name: timing_info_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.timing_info_id_seq OWNED BY public.timing_info.id;


--
-- Name: token_symbols; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.token_symbols (
    id integer NOT NULL,
    value text NOT NULL
);


ALTER TABLE public.token_symbols OWNER TO postgres;

--
-- Name: token_symbols_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.token_symbols_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.token_symbols_id_seq OWNER TO postgres;

--
-- Name: token_symbols_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.token_symbols_id_seq OWNED BY public.token_symbols.id;


--
-- Name: tokens; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.tokens (
    id integer NOT NULL,
    value text NOT NULL,
    owner_public_key_id integer,
    owner_token_id integer
);


ALTER TABLE public.tokens OWNER TO postgres;

--
-- Name: tokens_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.tokens_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.tokens_id_seq OWNER TO postgres;

--
-- Name: tokens_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.tokens_id_seq OWNED BY public.tokens.id;


--
-- Name: user_commands; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.user_commands (
    id integer NOT NULL,
    command_type public.user_command_type NOT NULL,
    fee_payer_id integer NOT NULL,
    source_id integer NOT NULL,
    receiver_id integer NOT NULL,
    nonce bigint NOT NULL,
    amount text,
    fee text NOT NULL,
    valid_until bigint,
    memo text NOT NULL,
    hash text NOT NULL
);


ALTER TABLE public.user_commands OWNER TO postgres;

--
-- Name: user_commands_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.user_commands_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.user_commands_id_seq OWNER TO postgres;

--
-- Name: user_commands_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.user_commands_id_seq OWNED BY public.user_commands.id;


--
-- Name: voting_for; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.voting_for (
    id integer NOT NULL,
    value text NOT NULL
);


ALTER TABLE public.voting_for OWNER TO postgres;

--
-- Name: voting_for_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.voting_for_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.voting_for_id_seq OWNER TO postgres;

--
-- Name: voting_for_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.voting_for_id_seq OWNED BY public.voting_for.id;


--
-- Name: zkapp_account_precondition; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_account_precondition (
    id integer NOT NULL,
    balance_id integer,
    nonce_id integer,
    receipt_chain_hash text,
    delegate_id integer,
    state_id integer NOT NULL,
    action_state_id integer,
    proved_state boolean,
    is_new boolean
);


ALTER TABLE public.zkapp_account_precondition OWNER TO postgres;

--
-- Name: zkapp_account_precondition_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_account_precondition_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_account_precondition_id_seq OWNER TO postgres;

--
-- Name: zkapp_account_precondition_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_account_precondition_id_seq OWNED BY public.zkapp_account_precondition.id;


--
-- Name: zkapp_account_update; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_account_update (
    id integer NOT NULL,
    body_id integer NOT NULL
);


ALTER TABLE public.zkapp_account_update OWNER TO postgres;

--
-- Name: zkapp_account_update_body; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_account_update_body (
    id integer NOT NULL,
    account_identifier_id integer NOT NULL,
    update_id integer NOT NULL,
    balance_change text NOT NULL,
    increment_nonce boolean NOT NULL,
    events_id integer NOT NULL,
    actions_id integer NOT NULL,
    call_data_id integer NOT NULL,
    call_depth integer NOT NULL,
    zkapp_network_precondition_id integer NOT NULL,
    zkapp_account_precondition_id integer NOT NULL,
    zkapp_valid_while_precondition_id integer,
    use_full_commitment boolean NOT NULL,
    implicit_account_creation_fee boolean NOT NULL,
    may_use_token public.may_use_token NOT NULL,
    authorization_kind public.authorization_kind_type NOT NULL,
    verification_key_hash_id integer
);


ALTER TABLE public.zkapp_account_update_body OWNER TO postgres;

--
-- Name: zkapp_account_update_body_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_account_update_body_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_account_update_body_id_seq OWNER TO postgres;

--
-- Name: zkapp_account_update_body_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_account_update_body_id_seq OWNED BY public.zkapp_account_update_body.id;


--
-- Name: zkapp_account_update_failures; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_account_update_failures (
    id integer NOT NULL,
    index integer NOT NULL,
    failures text[] NOT NULL
);


ALTER TABLE public.zkapp_account_update_failures OWNER TO postgres;

--
-- Name: zkapp_account_update_failures_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_account_update_failures_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_account_update_failures_id_seq OWNER TO postgres;

--
-- Name: zkapp_account_update_failures_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_account_update_failures_id_seq OWNED BY public.zkapp_account_update_failures.id;


--
-- Name: zkapp_account_update_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_account_update_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_account_update_id_seq OWNER TO postgres;

--
-- Name: zkapp_account_update_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_account_update_id_seq OWNED BY public.zkapp_account_update.id;


--
-- Name: zkapp_accounts; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_accounts (
    id integer NOT NULL,
    app_state_id integer NOT NULL,
    verification_key_id integer,
    zkapp_version bigint NOT NULL,
    action_state_id integer NOT NULL,
    last_action_slot bigint NOT NULL,
    proved_state boolean NOT NULL,
    zkapp_uri_id integer NOT NULL
);


ALTER TABLE public.zkapp_accounts OWNER TO postgres;

--
-- Name: zkapp_accounts_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_accounts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_accounts_id_seq OWNER TO postgres;

--
-- Name: zkapp_accounts_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_accounts_id_seq OWNED BY public.zkapp_accounts.id;


--
-- Name: zkapp_action_states; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_action_states (
    id integer NOT NULL,
    element0 integer NOT NULL,
    element1 integer NOT NULL,
    element2 integer NOT NULL,
    element3 integer NOT NULL,
    element4 integer NOT NULL
);


ALTER TABLE public.zkapp_action_states OWNER TO postgres;

--
-- Name: zkapp_action_states_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_action_states_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_action_states_id_seq OWNER TO postgres;

--
-- Name: zkapp_action_states_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_action_states_id_seq OWNED BY public.zkapp_action_states.id;


--
-- Name: zkapp_amount_bounds; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_amount_bounds (
    id integer NOT NULL,
    amount_lower_bound text NOT NULL,
    amount_upper_bound text NOT NULL
);


ALTER TABLE public.zkapp_amount_bounds OWNER TO postgres;

--
-- Name: zkapp_amount_bounds_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_amount_bounds_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_amount_bounds_id_seq OWNER TO postgres;

--
-- Name: zkapp_amount_bounds_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_amount_bounds_id_seq OWNED BY public.zkapp_amount_bounds.id;


--
-- Name: zkapp_balance_bounds; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_balance_bounds (
    id integer NOT NULL,
    balance_lower_bound text NOT NULL,
    balance_upper_bound text NOT NULL
);


ALTER TABLE public.zkapp_balance_bounds OWNER TO postgres;

--
-- Name: zkapp_balance_bounds_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_balance_bounds_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_balance_bounds_id_seq OWNER TO postgres;

--
-- Name: zkapp_balance_bounds_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_balance_bounds_id_seq OWNED BY public.zkapp_balance_bounds.id;


--
-- Name: zkapp_commands; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_commands (
    id integer NOT NULL,
    zkapp_fee_payer_body_id integer NOT NULL,
    zkapp_account_updates_ids integer[] NOT NULL,
    memo text NOT NULL,
    hash text NOT NULL
);


ALTER TABLE public.zkapp_commands OWNER TO postgres;

--
-- Name: zkapp_commands_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_commands_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_commands_id_seq OWNER TO postgres;

--
-- Name: zkapp_commands_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_commands_id_seq OWNED BY public.zkapp_commands.id;


--
-- Name: zkapp_epoch_data; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_epoch_data (
    id integer NOT NULL,
    epoch_ledger_id integer,
    epoch_seed text,
    start_checkpoint text,
    lock_checkpoint text,
    epoch_length_id integer
);


ALTER TABLE public.zkapp_epoch_data OWNER TO postgres;

--
-- Name: zkapp_epoch_data_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_epoch_data_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_epoch_data_id_seq OWNER TO postgres;

--
-- Name: zkapp_epoch_data_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_epoch_data_id_seq OWNED BY public.zkapp_epoch_data.id;


--
-- Name: zkapp_epoch_ledger; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_epoch_ledger (
    id integer NOT NULL,
    hash_id integer,
    total_currency_id integer
);


ALTER TABLE public.zkapp_epoch_ledger OWNER TO postgres;

--
-- Name: zkapp_epoch_ledger_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_epoch_ledger_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_epoch_ledger_id_seq OWNER TO postgres;

--
-- Name: zkapp_epoch_ledger_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_epoch_ledger_id_seq OWNED BY public.zkapp_epoch_ledger.id;


--
-- Name: zkapp_events; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_events (
    id integer NOT NULL,
    element_ids integer[] NOT NULL
);


ALTER TABLE public.zkapp_events OWNER TO postgres;

--
-- Name: zkapp_events_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_events_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_events_id_seq OWNER TO postgres;

--
-- Name: zkapp_events_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_events_id_seq OWNED BY public.zkapp_events.id;


--
-- Name: zkapp_fee_payer_body; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_fee_payer_body (
    id integer NOT NULL,
    public_key_id integer NOT NULL,
    fee text NOT NULL,
    valid_until bigint,
    nonce bigint NOT NULL
);


ALTER TABLE public.zkapp_fee_payer_body OWNER TO postgres;

--
-- Name: zkapp_fee_payer_body_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_fee_payer_body_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_fee_payer_body_id_seq OWNER TO postgres;

--
-- Name: zkapp_fee_payer_body_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_fee_payer_body_id_seq OWNED BY public.zkapp_fee_payer_body.id;


--
-- Name: zkapp_field; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_field (
    id integer NOT NULL,
    field text NOT NULL
);


ALTER TABLE public.zkapp_field OWNER TO postgres;

--
-- Name: zkapp_field_array; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_field_array (
    id integer NOT NULL,
    element_ids integer[] NOT NULL
);


ALTER TABLE public.zkapp_field_array OWNER TO postgres;

--
-- Name: zkapp_field_array_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_field_array_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_field_array_id_seq OWNER TO postgres;

--
-- Name: zkapp_field_array_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_field_array_id_seq OWNED BY public.zkapp_field_array.id;


--
-- Name: zkapp_field_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_field_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_field_id_seq OWNER TO postgres;

--
-- Name: zkapp_field_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_field_id_seq OWNED BY public.zkapp_field.id;


--
-- Name: zkapp_global_slot_bounds; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_global_slot_bounds (
    id integer NOT NULL,
    global_slot_lower_bound bigint NOT NULL,
    global_slot_upper_bound bigint NOT NULL
);


ALTER TABLE public.zkapp_global_slot_bounds OWNER TO postgres;

--
-- Name: zkapp_global_slot_bounds_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_global_slot_bounds_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_global_slot_bounds_id_seq OWNER TO postgres;

--
-- Name: zkapp_global_slot_bounds_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_global_slot_bounds_id_seq OWNED BY public.zkapp_global_slot_bounds.id;


--
-- Name: zkapp_length_bounds; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_length_bounds (
    id integer NOT NULL,
    length_lower_bound bigint NOT NULL,
    length_upper_bound bigint NOT NULL
);


ALTER TABLE public.zkapp_length_bounds OWNER TO postgres;

--
-- Name: zkapp_length_bounds_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_length_bounds_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_length_bounds_id_seq OWNER TO postgres;

--
-- Name: zkapp_length_bounds_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_length_bounds_id_seq OWNED BY public.zkapp_length_bounds.id;


--
-- Name: zkapp_network_precondition; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_network_precondition (
    id integer NOT NULL,
    snarked_ledger_hash_id integer,
    blockchain_length_id integer,
    min_window_density_id integer,
    total_currency_id integer,
    global_slot_since_genesis integer,
    staking_epoch_data_id integer,
    next_epoch_data_id integer
);


ALTER TABLE public.zkapp_network_precondition OWNER TO postgres;

--
-- Name: zkapp_network_precondition_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_network_precondition_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_network_precondition_id_seq OWNER TO postgres;

--
-- Name: zkapp_network_precondition_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_network_precondition_id_seq OWNED BY public.zkapp_network_precondition.id;


--
-- Name: zkapp_nonce_bounds; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_nonce_bounds (
    id integer NOT NULL,
    nonce_lower_bound bigint NOT NULL,
    nonce_upper_bound bigint NOT NULL
);


ALTER TABLE public.zkapp_nonce_bounds OWNER TO postgres;

--
-- Name: zkapp_nonce_bounds_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_nonce_bounds_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_nonce_bounds_id_seq OWNER TO postgres;

--
-- Name: zkapp_nonce_bounds_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_nonce_bounds_id_seq OWNED BY public.zkapp_nonce_bounds.id;


--
-- Name: zkapp_permissions; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_permissions (
    id integer NOT NULL,
    edit_state public.zkapp_auth_required_type NOT NULL,
    send public.zkapp_auth_required_type NOT NULL,
    receive public.zkapp_auth_required_type NOT NULL,
    access public.zkapp_auth_required_type NOT NULL,
    set_delegate public.zkapp_auth_required_type NOT NULL,
    set_permissions public.zkapp_auth_required_type NOT NULL,
    set_verification_key_auth public.zkapp_auth_required_type NOT NULL,
    set_verification_key_txn_version integer NOT NULL,
    set_zkapp_uri public.zkapp_auth_required_type NOT NULL,
    edit_action_state public.zkapp_auth_required_type NOT NULL,
    set_token_symbol public.zkapp_auth_required_type NOT NULL,
    increment_nonce public.zkapp_auth_required_type NOT NULL,
    set_voting_for public.zkapp_auth_required_type NOT NULL,
    set_timing public.zkapp_auth_required_type NOT NULL
);


ALTER TABLE public.zkapp_permissions OWNER TO postgres;

--
-- Name: zkapp_permissions_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_permissions_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_permissions_id_seq OWNER TO postgres;

--
-- Name: zkapp_permissions_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_permissions_id_seq OWNED BY public.zkapp_permissions.id;


--
-- Name: zkapp_states; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_states (
    id integer NOT NULL,
    element0 integer NOT NULL,
    element1 integer NOT NULL,
    element2 integer NOT NULL,
    element3 integer NOT NULL,
    element4 integer NOT NULL,
    element5 integer NOT NULL,
    element6 integer NOT NULL,
    element7 integer NOT NULL
);


ALTER TABLE public.zkapp_states OWNER TO postgres;

--
-- Name: zkapp_states_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_states_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_states_id_seq OWNER TO postgres;

--
-- Name: zkapp_states_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_states_id_seq OWNED BY public.zkapp_states.id;


--
-- Name: zkapp_states_nullable; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_states_nullable (
    id integer NOT NULL,
    element0 integer,
    element1 integer,
    element2 integer,
    element3 integer,
    element4 integer,
    element5 integer,
    element6 integer,
    element7 integer
);


ALTER TABLE public.zkapp_states_nullable OWNER TO postgres;

--
-- Name: zkapp_states_nullable_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_states_nullable_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_states_nullable_id_seq OWNER TO postgres;

--
-- Name: zkapp_states_nullable_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_states_nullable_id_seq OWNED BY public.zkapp_states_nullable.id;


--
-- Name: zkapp_timing_info; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_timing_info (
    id integer NOT NULL,
    initial_minimum_balance text NOT NULL,
    cliff_time bigint NOT NULL,
    cliff_amount text NOT NULL,
    vesting_period bigint NOT NULL,
    vesting_increment text NOT NULL
);


ALTER TABLE public.zkapp_timing_info OWNER TO postgres;

--
-- Name: zkapp_timing_info_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_timing_info_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_timing_info_id_seq OWNER TO postgres;

--
-- Name: zkapp_timing_info_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_timing_info_id_seq OWNED BY public.zkapp_timing_info.id;


--
-- Name: zkapp_token_id_bounds; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_token_id_bounds (
    id integer NOT NULL,
    token_id_lower_bound text NOT NULL,
    token_id_upper_bound text NOT NULL
);


ALTER TABLE public.zkapp_token_id_bounds OWNER TO postgres;

--
-- Name: zkapp_token_id_bounds_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_token_id_bounds_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_token_id_bounds_id_seq OWNER TO postgres;

--
-- Name: zkapp_token_id_bounds_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_token_id_bounds_id_seq OWNED BY public.zkapp_token_id_bounds.id;


--
-- Name: zkapp_updates; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_updates (
    id integer NOT NULL,
    app_state_id integer NOT NULL,
    delegate_id integer,
    verification_key_id integer,
    permissions_id integer,
    zkapp_uri_id integer,
    token_symbol_id integer,
    timing_id integer,
    voting_for_id integer
);


ALTER TABLE public.zkapp_updates OWNER TO postgres;

--
-- Name: zkapp_updates_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_updates_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_updates_id_seq OWNER TO postgres;

--
-- Name: zkapp_updates_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_updates_id_seq OWNED BY public.zkapp_updates.id;


--
-- Name: zkapp_uris; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_uris (
    id integer NOT NULL,
    value text NOT NULL
);


ALTER TABLE public.zkapp_uris OWNER TO postgres;

--
-- Name: zkapp_uris_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_uris_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_uris_id_seq OWNER TO postgres;

--
-- Name: zkapp_uris_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_uris_id_seq OWNED BY public.zkapp_uris.id;


--
-- Name: zkapp_verification_key_hashes; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_verification_key_hashes (
    id integer NOT NULL,
    value text NOT NULL
);


ALTER TABLE public.zkapp_verification_key_hashes OWNER TO postgres;

--
-- Name: zkapp_verification_key_hashes_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_verification_key_hashes_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_verification_key_hashes_id_seq OWNER TO postgres;

--
-- Name: zkapp_verification_key_hashes_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_verification_key_hashes_id_seq OWNED BY public.zkapp_verification_key_hashes.id;


--
-- Name: zkapp_verification_keys; Type: TABLE; Schema: public; Owner: postgres
--

CREATE TABLE public.zkapp_verification_keys (
    id integer NOT NULL,
    verification_key text NOT NULL,
    hash_id integer NOT NULL
);


ALTER TABLE public.zkapp_verification_keys OWNER TO postgres;

--
-- Name: zkapp_verification_keys_id_seq; Type: SEQUENCE; Schema: public; Owner: postgres
--

CREATE SEQUENCE public.zkapp_verification_keys_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;


ALTER SEQUENCE public.zkapp_verification_keys_id_seq OWNER TO postgres;

--
-- Name: zkapp_verification_keys_id_seq; Type: SEQUENCE OWNED BY; Schema: public; Owner: postgres
--

ALTER SEQUENCE public.zkapp_verification_keys_id_seq OWNED BY public.zkapp_verification_keys.id;


--
-- Name: account_identifiers id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.account_identifiers ALTER COLUMN id SET DEFAULT nextval('public.account_identifiers_id_seq'::regclass);


--
-- Name: blocks id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks ALTER COLUMN id SET DEFAULT nextval('public.blocks_id_seq'::regclass);


--
-- Name: epoch_data id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.epoch_data ALTER COLUMN id SET DEFAULT nextval('public.epoch_data_id_seq'::regclass);


--
-- Name: internal_commands id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.internal_commands ALTER COLUMN id SET DEFAULT nextval('public.internal_commands_id_seq'::regclass);


--
-- Name: protocol_versions id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.protocol_versions ALTER COLUMN id SET DEFAULT nextval('public.protocol_versions_id_seq'::regclass);


--
-- Name: public_keys id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.public_keys ALTER COLUMN id SET DEFAULT nextval('public.public_keys_id_seq'::regclass);


--
-- Name: snarked_ledger_hashes id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.snarked_ledger_hashes ALTER COLUMN id SET DEFAULT nextval('public.snarked_ledger_hashes_id_seq'::regclass);


--
-- Name: timing_info id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.timing_info ALTER COLUMN id SET DEFAULT nextval('public.timing_info_id_seq'::regclass);


--
-- Name: token_symbols id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.token_symbols ALTER COLUMN id SET DEFAULT nextval('public.token_symbols_id_seq'::regclass);


--
-- Name: tokens id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tokens ALTER COLUMN id SET DEFAULT nextval('public.tokens_id_seq'::regclass);


--
-- Name: user_commands id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_commands ALTER COLUMN id SET DEFAULT nextval('public.user_commands_id_seq'::regclass);


--
-- Name: voting_for id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.voting_for ALTER COLUMN id SET DEFAULT nextval('public.voting_for_id_seq'::regclass);


--
-- Name: zkapp_account_precondition id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_precondition ALTER COLUMN id SET DEFAULT nextval('public.zkapp_account_precondition_id_seq'::regclass);


--
-- Name: zkapp_account_update id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update ALTER COLUMN id SET DEFAULT nextval('public.zkapp_account_update_id_seq'::regclass);


--
-- Name: zkapp_account_update_body id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_body ALTER COLUMN id SET DEFAULT nextval('public.zkapp_account_update_body_id_seq'::regclass);


--
-- Name: zkapp_account_update_failures id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_failures ALTER COLUMN id SET DEFAULT nextval('public.zkapp_account_update_failures_id_seq'::regclass);


--
-- Name: zkapp_accounts id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_accounts ALTER COLUMN id SET DEFAULT nextval('public.zkapp_accounts_id_seq'::regclass);


--
-- Name: zkapp_action_states id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_action_states ALTER COLUMN id SET DEFAULT nextval('public.zkapp_action_states_id_seq'::regclass);


--
-- Name: zkapp_amount_bounds id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_amount_bounds ALTER COLUMN id SET DEFAULT nextval('public.zkapp_amount_bounds_id_seq'::regclass);


--
-- Name: zkapp_balance_bounds id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_balance_bounds ALTER COLUMN id SET DEFAULT nextval('public.zkapp_balance_bounds_id_seq'::regclass);


--
-- Name: zkapp_commands id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_commands ALTER COLUMN id SET DEFAULT nextval('public.zkapp_commands_id_seq'::regclass);


--
-- Name: zkapp_epoch_data id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_epoch_data ALTER COLUMN id SET DEFAULT nextval('public.zkapp_epoch_data_id_seq'::regclass);


--
-- Name: zkapp_epoch_ledger id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_epoch_ledger ALTER COLUMN id SET DEFAULT nextval('public.zkapp_epoch_ledger_id_seq'::regclass);


--
-- Name: zkapp_events id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_events ALTER COLUMN id SET DEFAULT nextval('public.zkapp_events_id_seq'::regclass);


--
-- Name: zkapp_fee_payer_body id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_fee_payer_body ALTER COLUMN id SET DEFAULT nextval('public.zkapp_fee_payer_body_id_seq'::regclass);


--
-- Name: zkapp_field id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_field ALTER COLUMN id SET DEFAULT nextval('public.zkapp_field_id_seq'::regclass);


--
-- Name: zkapp_field_array id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_field_array ALTER COLUMN id SET DEFAULT nextval('public.zkapp_field_array_id_seq'::regclass);


--
-- Name: zkapp_global_slot_bounds id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_global_slot_bounds ALTER COLUMN id SET DEFAULT nextval('public.zkapp_global_slot_bounds_id_seq'::regclass);


--
-- Name: zkapp_length_bounds id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_length_bounds ALTER COLUMN id SET DEFAULT nextval('public.zkapp_length_bounds_id_seq'::regclass);


--
-- Name: zkapp_network_precondition id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_network_precondition ALTER COLUMN id SET DEFAULT nextval('public.zkapp_network_precondition_id_seq'::regclass);


--
-- Name: zkapp_nonce_bounds id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_nonce_bounds ALTER COLUMN id SET DEFAULT nextval('public.zkapp_nonce_bounds_id_seq'::regclass);


--
-- Name: zkapp_permissions id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_permissions ALTER COLUMN id SET DEFAULT nextval('public.zkapp_permissions_id_seq'::regclass);


--
-- Name: zkapp_states id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states ALTER COLUMN id SET DEFAULT nextval('public.zkapp_states_id_seq'::regclass);


--
-- Name: zkapp_states_nullable id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states_nullable ALTER COLUMN id SET DEFAULT nextval('public.zkapp_states_nullable_id_seq'::regclass);


--
-- Name: zkapp_timing_info id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_timing_info ALTER COLUMN id SET DEFAULT nextval('public.zkapp_timing_info_id_seq'::regclass);


--
-- Name: zkapp_token_id_bounds id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_token_id_bounds ALTER COLUMN id SET DEFAULT nextval('public.zkapp_token_id_bounds_id_seq'::regclass);


--
-- Name: zkapp_updates id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_updates ALTER COLUMN id SET DEFAULT nextval('public.zkapp_updates_id_seq'::regclass);


--
-- Name: zkapp_uris id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_uris ALTER COLUMN id SET DEFAULT nextval('public.zkapp_uris_id_seq'::regclass);


--
-- Name: zkapp_verification_key_hashes id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_verification_key_hashes ALTER COLUMN id SET DEFAULT nextval('public.zkapp_verification_key_hashes_id_seq'::regclass);


--
-- Name: zkapp_verification_keys id; Type: DEFAULT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_verification_keys ALTER COLUMN id SET DEFAULT nextval('public.zkapp_verification_keys_id_seq'::regclass);


--
-- Name: account_identifiers account_identifiers_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.account_identifiers
    ADD CONSTRAINT account_identifiers_pkey PRIMARY KEY (id);


--
-- Name: account_identifiers account_identifiers_public_key_id_token_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.account_identifiers
    ADD CONSTRAINT account_identifiers_public_key_id_token_id_key UNIQUE (public_key_id, token_id);


--
-- Name: accounts_accessed accounts_accessed_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts_accessed
    ADD CONSTRAINT accounts_accessed_pkey PRIMARY KEY (block_id, account_identifier_id);


--
-- Name: accounts_created accounts_created_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts_created
    ADD CONSTRAINT accounts_created_pkey PRIMARY KEY (block_id, account_identifier_id);


--
-- Name: blocks_internal_commands blocks_internal_commands_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks_internal_commands
    ADD CONSTRAINT blocks_internal_commands_pkey PRIMARY KEY (block_id, internal_command_id, sequence_no, secondary_sequence_no);


--
-- Name: blocks blocks_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_pkey PRIMARY KEY (id);


--
-- Name: blocks blocks_state_hash_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_state_hash_key UNIQUE (state_hash);


--
-- Name: blocks_user_commands blocks_user_commands_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks_user_commands
    ADD CONSTRAINT blocks_user_commands_pkey PRIMARY KEY (block_id, user_command_id, sequence_no);


--
-- Name: blocks_zkapp_commands blocks_zkapp_commands_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks_zkapp_commands
    ADD CONSTRAINT blocks_zkapp_commands_pkey PRIMARY KEY (block_id, zkapp_command_id, sequence_no);


--
-- Name: epoch_data epoch_data_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.epoch_data
    ADD CONSTRAINT epoch_data_pkey PRIMARY KEY (id);


--
-- Name: epoch_data epoch_data_seed_ledger_hash_id_total_currency_start_checkpo_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.epoch_data
    ADD CONSTRAINT epoch_data_seed_ledger_hash_id_total_currency_start_checkpo_key UNIQUE (seed, ledger_hash_id, total_currency, start_checkpoint, lock_checkpoint, epoch_length);


--
-- Name: internal_commands internal_commands_hash_command_type_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.internal_commands
    ADD CONSTRAINT internal_commands_hash_command_type_key UNIQUE (hash, command_type);


--
-- Name: internal_commands internal_commands_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.internal_commands
    ADD CONSTRAINT internal_commands_pkey PRIMARY KEY (id);


--
-- Name: protocol_versions protocol_versions_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.protocol_versions
    ADD CONSTRAINT protocol_versions_pkey PRIMARY KEY (id);


--
-- Name: protocol_versions protocol_versions_transaction_network_patch_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.protocol_versions
    ADD CONSTRAINT protocol_versions_transaction_network_patch_key UNIQUE (transaction, network, patch);


--
-- Name: public_keys public_keys_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.public_keys
    ADD CONSTRAINT public_keys_pkey PRIMARY KEY (id);


--
-- Name: public_keys public_keys_value_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.public_keys
    ADD CONSTRAINT public_keys_value_key UNIQUE (value);


--
-- Name: snarked_ledger_hashes snarked_ledger_hashes_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.snarked_ledger_hashes
    ADD CONSTRAINT snarked_ledger_hashes_pkey PRIMARY KEY (id);


--
-- Name: snarked_ledger_hashes snarked_ledger_hashes_value_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.snarked_ledger_hashes
    ADD CONSTRAINT snarked_ledger_hashes_value_key UNIQUE (value);


--
-- Name: timing_info timing_info_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.timing_info
    ADD CONSTRAINT timing_info_pkey PRIMARY KEY (id);


--
-- Name: token_symbols token_symbols_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.token_symbols
    ADD CONSTRAINT token_symbols_pkey PRIMARY KEY (id);


--
-- Name: tokens tokens_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tokens
    ADD CONSTRAINT tokens_pkey PRIMARY KEY (id);


--
-- Name: tokens tokens_value_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tokens
    ADD CONSTRAINT tokens_value_key UNIQUE (value);


--
-- Name: user_commands user_commands_hash_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_commands
    ADD CONSTRAINT user_commands_hash_key UNIQUE (hash);


--
-- Name: user_commands user_commands_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_commands
    ADD CONSTRAINT user_commands_pkey PRIMARY KEY (id);


--
-- Name: voting_for voting_for_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.voting_for
    ADD CONSTRAINT voting_for_pkey PRIMARY KEY (id);


--
-- Name: zkapp_account_precondition zkapp_account_precondition_balance_id_receipt_chain_hash_de_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_precondition
    ADD CONSTRAINT zkapp_account_precondition_balance_id_receipt_chain_hash_de_key UNIQUE (balance_id, receipt_chain_hash, delegate_id, state_id, action_state_id, proved_state, is_new, nonce_id);


--
-- Name: zkapp_account_precondition zkapp_account_precondition_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_precondition
    ADD CONSTRAINT zkapp_account_precondition_pkey PRIMARY KEY (id);


--
-- Name: zkapp_account_update_body zkapp_account_update_body_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_body
    ADD CONSTRAINT zkapp_account_update_body_pkey PRIMARY KEY (id);


--
-- Name: zkapp_account_update_failures zkapp_account_update_failures_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_failures
    ADD CONSTRAINT zkapp_account_update_failures_pkey PRIMARY KEY (id);


--
-- Name: zkapp_account_update zkapp_account_update_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update
    ADD CONSTRAINT zkapp_account_update_pkey PRIMARY KEY (id);


--
-- Name: zkapp_accounts zkapp_accounts_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_accounts
    ADD CONSTRAINT zkapp_accounts_pkey PRIMARY KEY (id);


--
-- Name: zkapp_action_states zkapp_action_states_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_action_states
    ADD CONSTRAINT zkapp_action_states_pkey PRIMARY KEY (id);


--
-- Name: zkapp_amount_bounds zkapp_amount_bounds_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_amount_bounds
    ADD CONSTRAINT zkapp_amount_bounds_pkey PRIMARY KEY (id);


--
-- Name: zkapp_balance_bounds zkapp_balance_bounds_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_balance_bounds
    ADD CONSTRAINT zkapp_balance_bounds_pkey PRIMARY KEY (id);


--
-- Name: zkapp_commands zkapp_commands_hash_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_commands
    ADD CONSTRAINT zkapp_commands_hash_key UNIQUE (hash);


--
-- Name: zkapp_commands zkapp_commands_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_commands
    ADD CONSTRAINT zkapp_commands_pkey PRIMARY KEY (id);


--
-- Name: zkapp_epoch_data zkapp_epoch_data_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_epoch_data
    ADD CONSTRAINT zkapp_epoch_data_pkey PRIMARY KEY (id);


--
-- Name: zkapp_epoch_ledger zkapp_epoch_ledger_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_epoch_ledger
    ADD CONSTRAINT zkapp_epoch_ledger_pkey PRIMARY KEY (id);


--
-- Name: zkapp_events zkapp_events_element_ids_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_events
    ADD CONSTRAINT zkapp_events_element_ids_key UNIQUE (element_ids);


--
-- Name: zkapp_events zkapp_events_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_events
    ADD CONSTRAINT zkapp_events_pkey PRIMARY KEY (id);


--
-- Name: zkapp_fee_payer_body zkapp_fee_payer_body_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_fee_payer_body
    ADD CONSTRAINT zkapp_fee_payer_body_pkey PRIMARY KEY (id);


--
-- Name: zkapp_field_array zkapp_field_array_element_ids_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_field_array
    ADD CONSTRAINT zkapp_field_array_element_ids_key UNIQUE (element_ids);


--
-- Name: zkapp_field_array zkapp_field_array_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_field_array
    ADD CONSTRAINT zkapp_field_array_pkey PRIMARY KEY (id);


--
-- Name: zkapp_field zkapp_field_field_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_field
    ADD CONSTRAINT zkapp_field_field_key UNIQUE (field);


--
-- Name: zkapp_field zkapp_field_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_field
    ADD CONSTRAINT zkapp_field_pkey PRIMARY KEY (id);


--
-- Name: zkapp_global_slot_bounds zkapp_global_slot_bounds_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_global_slot_bounds
    ADD CONSTRAINT zkapp_global_slot_bounds_pkey PRIMARY KEY (id);


--
-- Name: zkapp_length_bounds zkapp_length_bounds_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_length_bounds
    ADD CONSTRAINT zkapp_length_bounds_pkey PRIMARY KEY (id);


--
-- Name: zkapp_network_precondition zkapp_network_precondition_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_network_precondition
    ADD CONSTRAINT zkapp_network_precondition_pkey PRIMARY KEY (id);


--
-- Name: zkapp_nonce_bounds zkapp_nonce_bounds_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_nonce_bounds
    ADD CONSTRAINT zkapp_nonce_bounds_pkey PRIMARY KEY (id);


--
-- Name: zkapp_permissions zkapp_permissions_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_permissions
    ADD CONSTRAINT zkapp_permissions_pkey PRIMARY KEY (id);


--
-- Name: zkapp_states_nullable zkapp_states_nullable_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states_nullable
    ADD CONSTRAINT zkapp_states_nullable_pkey PRIMARY KEY (id);


--
-- Name: zkapp_states zkapp_states_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states
    ADD CONSTRAINT zkapp_states_pkey PRIMARY KEY (id);


--
-- Name: zkapp_timing_info zkapp_timing_info_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_timing_info
    ADD CONSTRAINT zkapp_timing_info_pkey PRIMARY KEY (id);


--
-- Name: zkapp_token_id_bounds zkapp_token_id_bounds_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_token_id_bounds
    ADD CONSTRAINT zkapp_token_id_bounds_pkey PRIMARY KEY (id);


--
-- Name: zkapp_updates zkapp_updates_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_updates
    ADD CONSTRAINT zkapp_updates_pkey PRIMARY KEY (id);


--
-- Name: zkapp_uris zkapp_uris_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_uris
    ADD CONSTRAINT zkapp_uris_pkey PRIMARY KEY (id);


--
-- Name: zkapp_uris zkapp_uris_value_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_uris
    ADD CONSTRAINT zkapp_uris_value_key UNIQUE (value);


--
-- Name: zkapp_verification_key_hashes zkapp_verification_key_hashes_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_verification_key_hashes
    ADD CONSTRAINT zkapp_verification_key_hashes_pkey PRIMARY KEY (id);


--
-- Name: zkapp_verification_key_hashes zkapp_verification_key_hashes_value_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_verification_key_hashes
    ADD CONSTRAINT zkapp_verification_key_hashes_value_key UNIQUE (value);


--
-- Name: zkapp_verification_keys zkapp_verification_keys_hash_id_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_verification_keys
    ADD CONSTRAINT zkapp_verification_keys_hash_id_key UNIQUE (hash_id);


--
-- Name: zkapp_verification_keys zkapp_verification_keys_pkey; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_verification_keys
    ADD CONSTRAINT zkapp_verification_keys_pkey PRIMARY KEY (id);


--
-- Name: zkapp_verification_keys zkapp_verification_keys_verification_key_key; Type: CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_verification_keys
    ADD CONSTRAINT zkapp_verification_keys_verification_key_key UNIQUE (verification_key);


--
-- Name: idx_accounts_accessed_block_account_identifier_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_accounts_accessed_block_account_identifier_id ON public.accounts_accessed USING btree (account_identifier_id);


--
-- Name: idx_accounts_accessed_block_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_accounts_accessed_block_id ON public.accounts_accessed USING btree (block_id);


--
-- Name: idx_accounts_created_block_account_identifier_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_accounts_created_block_account_identifier_id ON public.accounts_created USING btree (account_identifier_id);


--
-- Name: idx_accounts_created_block_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_accounts_created_block_id ON public.accounts_created USING btree (block_id);


--
-- Name: idx_blocks_creator_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_creator_id ON public.blocks USING btree (creator_id);


--
-- Name: idx_blocks_height; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_height ON public.blocks USING btree (height);


--
-- Name: idx_blocks_internal_commands_block_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_internal_commands_block_id ON public.blocks_internal_commands USING btree (block_id);


--
-- Name: idx_blocks_internal_commands_internal_command_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_internal_commands_internal_command_id ON public.blocks_internal_commands USING btree (internal_command_id);


--
-- Name: idx_blocks_internal_commands_secondary_sequence_no; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_internal_commands_secondary_sequence_no ON public.blocks_internal_commands USING btree (secondary_sequence_no);


--
-- Name: idx_blocks_internal_commands_sequence_no; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_internal_commands_sequence_no ON public.blocks_internal_commands USING btree (sequence_no);


--
-- Name: idx_blocks_parent_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_parent_id ON public.blocks USING btree (parent_id);


--
-- Name: idx_blocks_user_commands_block_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_user_commands_block_id ON public.blocks_user_commands USING btree (block_id);


--
-- Name: idx_blocks_user_commands_sequence_no; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_user_commands_sequence_no ON public.blocks_user_commands USING btree (sequence_no);


--
-- Name: idx_blocks_user_commands_user_command_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_user_commands_user_command_id ON public.blocks_user_commands USING btree (user_command_id);


--
-- Name: idx_blocks_zkapp_commands_block_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_zkapp_commands_block_id ON public.blocks_zkapp_commands USING btree (block_id);


--
-- Name: idx_blocks_zkapp_commands_sequence_no; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_zkapp_commands_sequence_no ON public.blocks_zkapp_commands USING btree (sequence_no);


--
-- Name: idx_blocks_zkapp_commands_zkapp_command_id; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_blocks_zkapp_commands_zkapp_command_id ON public.blocks_zkapp_commands USING btree (zkapp_command_id);


--
-- Name: idx_chain_status; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_chain_status ON public.blocks USING btree (chain_status);


--
-- Name: idx_token_symbols_value; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_token_symbols_value ON public.token_symbols USING btree (value);


--
-- Name: idx_voting_for_value; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_voting_for_value ON public.voting_for USING btree (value);


--
-- Name: idx_zkapp_events_element_ids; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_zkapp_events_element_ids ON public.zkapp_events USING btree (element_ids);


--
-- Name: idx_zkapp_field_array_element_ids; Type: INDEX; Schema: public; Owner: postgres
--

CREATE INDEX idx_zkapp_field_array_element_ids ON public.zkapp_field_array USING btree (element_ids);


--
-- Name: account_identifiers account_identifiers_public_key_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.account_identifiers
    ADD CONSTRAINT account_identifiers_public_key_id_fkey FOREIGN KEY (public_key_id) REFERENCES public.public_keys(id) ON DELETE CASCADE;


--
-- Name: account_identifiers account_identifiers_token_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.account_identifiers
    ADD CONSTRAINT account_identifiers_token_id_fkey FOREIGN KEY (token_id) REFERENCES public.tokens(id) ON DELETE CASCADE;


--
-- Name: accounts_accessed accounts_accessed_account_identifier_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts_accessed
    ADD CONSTRAINT accounts_accessed_account_identifier_id_fkey FOREIGN KEY (account_identifier_id) REFERENCES public.account_identifiers(id);


--
-- Name: accounts_accessed accounts_accessed_block_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts_accessed
    ADD CONSTRAINT accounts_accessed_block_id_fkey FOREIGN KEY (block_id) REFERENCES public.blocks(id);


--
-- Name: accounts_accessed accounts_accessed_delegate_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts_accessed
    ADD CONSTRAINT accounts_accessed_delegate_id_fkey FOREIGN KEY (delegate_id) REFERENCES public.public_keys(id);


--
-- Name: accounts_accessed accounts_accessed_permissions_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts_accessed
    ADD CONSTRAINT accounts_accessed_permissions_id_fkey FOREIGN KEY (permissions_id) REFERENCES public.zkapp_permissions(id);


--
-- Name: accounts_accessed accounts_accessed_timing_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts_accessed
    ADD CONSTRAINT accounts_accessed_timing_id_fkey FOREIGN KEY (timing_id) REFERENCES public.timing_info(id);


--
-- Name: accounts_accessed accounts_accessed_token_symbol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts_accessed
    ADD CONSTRAINT accounts_accessed_token_symbol_id_fkey FOREIGN KEY (token_symbol_id) REFERENCES public.token_symbols(id);


--
-- Name: accounts_accessed accounts_accessed_voting_for_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts_accessed
    ADD CONSTRAINT accounts_accessed_voting_for_id_fkey FOREIGN KEY (voting_for_id) REFERENCES public.voting_for(id);


--
-- Name: accounts_accessed accounts_accessed_zkapp_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts_accessed
    ADD CONSTRAINT accounts_accessed_zkapp_id_fkey FOREIGN KEY (zkapp_id) REFERENCES public.zkapp_accounts(id);


--
-- Name: accounts_created accounts_created_account_identifier_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts_created
    ADD CONSTRAINT accounts_created_account_identifier_id_fkey FOREIGN KEY (account_identifier_id) REFERENCES public.account_identifiers(id);


--
-- Name: accounts_created accounts_created_block_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.accounts_created
    ADD CONSTRAINT accounts_created_block_id_fkey FOREIGN KEY (block_id) REFERENCES public.blocks(id);


--
-- Name: blocks blocks_block_winner_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_block_winner_id_fkey FOREIGN KEY (block_winner_id) REFERENCES public.public_keys(id);


--
-- Name: blocks blocks_creator_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.public_keys(id);


--
-- Name: blocks_internal_commands blocks_internal_commands_block_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks_internal_commands
    ADD CONSTRAINT blocks_internal_commands_block_id_fkey FOREIGN KEY (block_id) REFERENCES public.blocks(id) ON DELETE CASCADE;


--
-- Name: blocks_internal_commands blocks_internal_commands_internal_command_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks_internal_commands
    ADD CONSTRAINT blocks_internal_commands_internal_command_id_fkey FOREIGN KEY (internal_command_id) REFERENCES public.internal_commands(id) ON DELETE CASCADE;


--
-- Name: blocks blocks_next_epoch_data_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_next_epoch_data_id_fkey FOREIGN KEY (next_epoch_data_id) REFERENCES public.epoch_data(id);


--
-- Name: blocks blocks_parent_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_parent_id_fkey FOREIGN KEY (parent_id) REFERENCES public.blocks(id);


--
-- Name: blocks blocks_proposed_protocol_version_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_proposed_protocol_version_id_fkey FOREIGN KEY (proposed_protocol_version_id) REFERENCES public.protocol_versions(id);


--
-- Name: blocks blocks_protocol_version_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_protocol_version_id_fkey FOREIGN KEY (protocol_version_id) REFERENCES public.protocol_versions(id);


--
-- Name: blocks blocks_snarked_ledger_hash_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_snarked_ledger_hash_id_fkey FOREIGN KEY (snarked_ledger_hash_id) REFERENCES public.snarked_ledger_hashes(id);


--
-- Name: blocks blocks_staking_epoch_data_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks
    ADD CONSTRAINT blocks_staking_epoch_data_id_fkey FOREIGN KEY (staking_epoch_data_id) REFERENCES public.epoch_data(id);


--
-- Name: blocks_user_commands blocks_user_commands_block_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks_user_commands
    ADD CONSTRAINT blocks_user_commands_block_id_fkey FOREIGN KEY (block_id) REFERENCES public.blocks(id) ON DELETE CASCADE;


--
-- Name: blocks_user_commands blocks_user_commands_user_command_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks_user_commands
    ADD CONSTRAINT blocks_user_commands_user_command_id_fkey FOREIGN KEY (user_command_id) REFERENCES public.user_commands(id) ON DELETE CASCADE;


--
-- Name: blocks_zkapp_commands blocks_zkapp_commands_block_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks_zkapp_commands
    ADD CONSTRAINT blocks_zkapp_commands_block_id_fkey FOREIGN KEY (block_id) REFERENCES public.blocks(id) ON DELETE CASCADE;


--
-- Name: blocks_zkapp_commands blocks_zkapp_commands_zkapp_command_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.blocks_zkapp_commands
    ADD CONSTRAINT blocks_zkapp_commands_zkapp_command_id_fkey FOREIGN KEY (zkapp_command_id) REFERENCES public.zkapp_commands(id) ON DELETE CASCADE;


--
-- Name: epoch_data epoch_data_ledger_hash_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.epoch_data
    ADD CONSTRAINT epoch_data_ledger_hash_id_fkey FOREIGN KEY (ledger_hash_id) REFERENCES public.snarked_ledger_hashes(id);


--
-- Name: internal_commands internal_commands_receiver_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.internal_commands
    ADD CONSTRAINT internal_commands_receiver_id_fkey FOREIGN KEY (receiver_id) REFERENCES public.public_keys(id);


--
-- Name: timing_info timing_info_account_identifier_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.timing_info
    ADD CONSTRAINT timing_info_account_identifier_id_fkey FOREIGN KEY (account_identifier_id) REFERENCES public.account_identifiers(id);


--
-- Name: tokens tokens_owner_public_key_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tokens
    ADD CONSTRAINT tokens_owner_public_key_id_fkey FOREIGN KEY (owner_public_key_id) REFERENCES public.public_keys(id) ON DELETE CASCADE;


--
-- Name: tokens tokens_owner_token_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.tokens
    ADD CONSTRAINT tokens_owner_token_id_fkey FOREIGN KEY (owner_token_id) REFERENCES public.tokens(id);


--
-- Name: user_commands user_commands_fee_payer_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_commands
    ADD CONSTRAINT user_commands_fee_payer_id_fkey FOREIGN KEY (fee_payer_id) REFERENCES public.public_keys(id);


--
-- Name: user_commands user_commands_receiver_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_commands
    ADD CONSTRAINT user_commands_receiver_id_fkey FOREIGN KEY (receiver_id) REFERENCES public.public_keys(id);


--
-- Name: user_commands user_commands_source_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.user_commands
    ADD CONSTRAINT user_commands_source_id_fkey FOREIGN KEY (source_id) REFERENCES public.public_keys(id);


--
-- Name: zkapp_account_precondition zkapp_account_precondition_action_state_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_precondition
    ADD CONSTRAINT zkapp_account_precondition_action_state_id_fkey FOREIGN KEY (action_state_id) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_account_precondition zkapp_account_precondition_balance_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_precondition
    ADD CONSTRAINT zkapp_account_precondition_balance_id_fkey FOREIGN KEY (balance_id) REFERENCES public.zkapp_balance_bounds(id);


--
-- Name: zkapp_account_precondition zkapp_account_precondition_delegate_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_precondition
    ADD CONSTRAINT zkapp_account_precondition_delegate_id_fkey FOREIGN KEY (delegate_id) REFERENCES public.public_keys(id);


--
-- Name: zkapp_account_precondition zkapp_account_precondition_nonce_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_precondition
    ADD CONSTRAINT zkapp_account_precondition_nonce_id_fkey FOREIGN KEY (nonce_id) REFERENCES public.zkapp_nonce_bounds(id);


--
-- Name: zkapp_account_precondition zkapp_account_precondition_state_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_precondition
    ADD CONSTRAINT zkapp_account_precondition_state_id_fkey FOREIGN KEY (state_id) REFERENCES public.zkapp_states_nullable(id);


--
-- Name: zkapp_account_update_body zkapp_account_update_body_account_identifier_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_body
    ADD CONSTRAINT zkapp_account_update_body_account_identifier_id_fkey FOREIGN KEY (account_identifier_id) REFERENCES public.account_identifiers(id);


--
-- Name: zkapp_account_update_body zkapp_account_update_body_actions_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_body
    ADD CONSTRAINT zkapp_account_update_body_actions_id_fkey FOREIGN KEY (actions_id) REFERENCES public.zkapp_events(id);


--
-- Name: zkapp_account_update_body zkapp_account_update_body_call_data_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_body
    ADD CONSTRAINT zkapp_account_update_body_call_data_id_fkey FOREIGN KEY (call_data_id) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_account_update_body zkapp_account_update_body_events_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_body
    ADD CONSTRAINT zkapp_account_update_body_events_id_fkey FOREIGN KEY (events_id) REFERENCES public.zkapp_events(id);


--
-- Name: zkapp_account_update zkapp_account_update_body_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update
    ADD CONSTRAINT zkapp_account_update_body_id_fkey FOREIGN KEY (body_id) REFERENCES public.zkapp_account_update_body(id);


--
-- Name: zkapp_account_update_body zkapp_account_update_body_update_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_body
    ADD CONSTRAINT zkapp_account_update_body_update_id_fkey FOREIGN KEY (update_id) REFERENCES public.zkapp_updates(id);


--
-- Name: zkapp_account_update_body zkapp_account_update_body_verification_key_hash_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_body
    ADD CONSTRAINT zkapp_account_update_body_verification_key_hash_id_fkey FOREIGN KEY (verification_key_hash_id) REFERENCES public.zkapp_verification_key_hashes(id);


--
-- Name: zkapp_account_update_body zkapp_account_update_body_zkapp_account_precondition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_body
    ADD CONSTRAINT zkapp_account_update_body_zkapp_account_precondition_id_fkey FOREIGN KEY (zkapp_account_precondition_id) REFERENCES public.zkapp_account_precondition(id);


--
-- Name: zkapp_account_update_body zkapp_account_update_body_zkapp_network_precondition_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_body
    ADD CONSTRAINT zkapp_account_update_body_zkapp_network_precondition_id_fkey FOREIGN KEY (zkapp_network_precondition_id) REFERENCES public.zkapp_network_precondition(id);


--
-- Name: zkapp_account_update_body zkapp_account_update_body_zkapp_valid_while_precondition_i_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_account_update_body
    ADD CONSTRAINT zkapp_account_update_body_zkapp_valid_while_precondition_i_fkey FOREIGN KEY (zkapp_valid_while_precondition_id) REFERENCES public.zkapp_global_slot_bounds(id);


--
-- Name: zkapp_accounts zkapp_accounts_action_state_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_accounts
    ADD CONSTRAINT zkapp_accounts_action_state_id_fkey FOREIGN KEY (action_state_id) REFERENCES public.zkapp_action_states(id);


--
-- Name: zkapp_accounts zkapp_accounts_app_state_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_accounts
    ADD CONSTRAINT zkapp_accounts_app_state_id_fkey FOREIGN KEY (app_state_id) REFERENCES public.zkapp_states(id);


--
-- Name: zkapp_accounts zkapp_accounts_verification_key_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_accounts
    ADD CONSTRAINT zkapp_accounts_verification_key_id_fkey FOREIGN KEY (verification_key_id) REFERENCES public.zkapp_verification_keys(id);


--
-- Name: zkapp_accounts zkapp_accounts_zkapp_uri_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_accounts
    ADD CONSTRAINT zkapp_accounts_zkapp_uri_id_fkey FOREIGN KEY (zkapp_uri_id) REFERENCES public.zkapp_uris(id);


--
-- Name: zkapp_action_states zkapp_action_states_element0_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_action_states
    ADD CONSTRAINT zkapp_action_states_element0_fkey FOREIGN KEY (element0) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_action_states zkapp_action_states_element1_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_action_states
    ADD CONSTRAINT zkapp_action_states_element1_fkey FOREIGN KEY (element1) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_action_states zkapp_action_states_element2_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_action_states
    ADD CONSTRAINT zkapp_action_states_element2_fkey FOREIGN KEY (element2) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_action_states zkapp_action_states_element3_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_action_states
    ADD CONSTRAINT zkapp_action_states_element3_fkey FOREIGN KEY (element3) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_action_states zkapp_action_states_element4_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_action_states
    ADD CONSTRAINT zkapp_action_states_element4_fkey FOREIGN KEY (element4) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_commands zkapp_commands_zkapp_fee_payer_body_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_commands
    ADD CONSTRAINT zkapp_commands_zkapp_fee_payer_body_id_fkey FOREIGN KEY (zkapp_fee_payer_body_id) REFERENCES public.zkapp_fee_payer_body(id);


--
-- Name: zkapp_epoch_data zkapp_epoch_data_epoch_ledger_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_epoch_data
    ADD CONSTRAINT zkapp_epoch_data_epoch_ledger_id_fkey FOREIGN KEY (epoch_ledger_id) REFERENCES public.zkapp_epoch_ledger(id);


--
-- Name: zkapp_epoch_data zkapp_epoch_data_epoch_length_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_epoch_data
    ADD CONSTRAINT zkapp_epoch_data_epoch_length_id_fkey FOREIGN KEY (epoch_length_id) REFERENCES public.zkapp_length_bounds(id);


--
-- Name: zkapp_epoch_ledger zkapp_epoch_ledger_hash_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_epoch_ledger
    ADD CONSTRAINT zkapp_epoch_ledger_hash_id_fkey FOREIGN KEY (hash_id) REFERENCES public.snarked_ledger_hashes(id);


--
-- Name: zkapp_epoch_ledger zkapp_epoch_ledger_total_currency_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_epoch_ledger
    ADD CONSTRAINT zkapp_epoch_ledger_total_currency_id_fkey FOREIGN KEY (total_currency_id) REFERENCES public.zkapp_amount_bounds(id);


--
-- Name: zkapp_fee_payer_body zkapp_fee_payer_body_public_key_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_fee_payer_body
    ADD CONSTRAINT zkapp_fee_payer_body_public_key_id_fkey FOREIGN KEY (public_key_id) REFERENCES public.public_keys(id);


--
-- Name: zkapp_network_precondition zkapp_network_precondition_blockchain_length_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_network_precondition
    ADD CONSTRAINT zkapp_network_precondition_blockchain_length_id_fkey FOREIGN KEY (blockchain_length_id) REFERENCES public.zkapp_length_bounds(id);


--
-- Name: zkapp_network_precondition zkapp_network_precondition_global_slot_since_genesis_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_network_precondition
    ADD CONSTRAINT zkapp_network_precondition_global_slot_since_genesis_fkey FOREIGN KEY (global_slot_since_genesis) REFERENCES public.zkapp_global_slot_bounds(id);


--
-- Name: zkapp_network_precondition zkapp_network_precondition_min_window_density_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_network_precondition
    ADD CONSTRAINT zkapp_network_precondition_min_window_density_id_fkey FOREIGN KEY (min_window_density_id) REFERENCES public.zkapp_length_bounds(id);


--
-- Name: zkapp_network_precondition zkapp_network_precondition_next_epoch_data_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_network_precondition
    ADD CONSTRAINT zkapp_network_precondition_next_epoch_data_id_fkey FOREIGN KEY (next_epoch_data_id) REFERENCES public.zkapp_epoch_data(id);


--
-- Name: zkapp_network_precondition zkapp_network_precondition_snarked_ledger_hash_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_network_precondition
    ADD CONSTRAINT zkapp_network_precondition_snarked_ledger_hash_id_fkey FOREIGN KEY (snarked_ledger_hash_id) REFERENCES public.snarked_ledger_hashes(id);


--
-- Name: zkapp_network_precondition zkapp_network_precondition_staking_epoch_data_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_network_precondition
    ADD CONSTRAINT zkapp_network_precondition_staking_epoch_data_id_fkey FOREIGN KEY (staking_epoch_data_id) REFERENCES public.zkapp_epoch_data(id);


--
-- Name: zkapp_network_precondition zkapp_network_precondition_total_currency_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_network_precondition
    ADD CONSTRAINT zkapp_network_precondition_total_currency_id_fkey FOREIGN KEY (total_currency_id) REFERENCES public.zkapp_amount_bounds(id);


--
-- Name: zkapp_states zkapp_states_element0_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states
    ADD CONSTRAINT zkapp_states_element0_fkey FOREIGN KEY (element0) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states zkapp_states_element1_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states
    ADD CONSTRAINT zkapp_states_element1_fkey FOREIGN KEY (element1) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states zkapp_states_element2_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states
    ADD CONSTRAINT zkapp_states_element2_fkey FOREIGN KEY (element2) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states zkapp_states_element3_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states
    ADD CONSTRAINT zkapp_states_element3_fkey FOREIGN KEY (element3) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states zkapp_states_element4_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states
    ADD CONSTRAINT zkapp_states_element4_fkey FOREIGN KEY (element4) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states zkapp_states_element5_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states
    ADD CONSTRAINT zkapp_states_element5_fkey FOREIGN KEY (element5) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states zkapp_states_element6_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states
    ADD CONSTRAINT zkapp_states_element6_fkey FOREIGN KEY (element6) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states zkapp_states_element7_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states
    ADD CONSTRAINT zkapp_states_element7_fkey FOREIGN KEY (element7) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states_nullable zkapp_states_nullable_element0_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states_nullable
    ADD CONSTRAINT zkapp_states_nullable_element0_fkey FOREIGN KEY (element0) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states_nullable zkapp_states_nullable_element1_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states_nullable
    ADD CONSTRAINT zkapp_states_nullable_element1_fkey FOREIGN KEY (element1) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states_nullable zkapp_states_nullable_element2_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states_nullable
    ADD CONSTRAINT zkapp_states_nullable_element2_fkey FOREIGN KEY (element2) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states_nullable zkapp_states_nullable_element3_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states_nullable
    ADD CONSTRAINT zkapp_states_nullable_element3_fkey FOREIGN KEY (element3) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states_nullable zkapp_states_nullable_element4_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states_nullable
    ADD CONSTRAINT zkapp_states_nullable_element4_fkey FOREIGN KEY (element4) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states_nullable zkapp_states_nullable_element5_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states_nullable
    ADD CONSTRAINT zkapp_states_nullable_element5_fkey FOREIGN KEY (element5) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states_nullable zkapp_states_nullable_element6_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states_nullable
    ADD CONSTRAINT zkapp_states_nullable_element6_fkey FOREIGN KEY (element6) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_states_nullable zkapp_states_nullable_element7_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_states_nullable
    ADD CONSTRAINT zkapp_states_nullable_element7_fkey FOREIGN KEY (element7) REFERENCES public.zkapp_field(id);


--
-- Name: zkapp_updates zkapp_updates_app_state_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_updates
    ADD CONSTRAINT zkapp_updates_app_state_id_fkey FOREIGN KEY (app_state_id) REFERENCES public.zkapp_states_nullable(id);


--
-- Name: zkapp_updates zkapp_updates_delegate_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_updates
    ADD CONSTRAINT zkapp_updates_delegate_id_fkey FOREIGN KEY (delegate_id) REFERENCES public.public_keys(id);


--
-- Name: zkapp_updates zkapp_updates_permissions_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_updates
    ADD CONSTRAINT zkapp_updates_permissions_id_fkey FOREIGN KEY (permissions_id) REFERENCES public.zkapp_permissions(id);


--
-- Name: zkapp_updates zkapp_updates_timing_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_updates
    ADD CONSTRAINT zkapp_updates_timing_id_fkey FOREIGN KEY (timing_id) REFERENCES public.zkapp_timing_info(id);


--
-- Name: zkapp_updates zkapp_updates_token_symbol_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_updates
    ADD CONSTRAINT zkapp_updates_token_symbol_id_fkey FOREIGN KEY (token_symbol_id) REFERENCES public.token_symbols(id);


--
-- Name: zkapp_updates zkapp_updates_verification_key_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_updates
    ADD CONSTRAINT zkapp_updates_verification_key_id_fkey FOREIGN KEY (verification_key_id) REFERENCES public.zkapp_verification_keys(id);


--
-- Name: zkapp_updates zkapp_updates_voting_for_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_updates
    ADD CONSTRAINT zkapp_updates_voting_for_id_fkey FOREIGN KEY (voting_for_id) REFERENCES public.voting_for(id);


--
-- Name: zkapp_updates zkapp_updates_zkapp_uri_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_updates
    ADD CONSTRAINT zkapp_updates_zkapp_uri_id_fkey FOREIGN KEY (zkapp_uri_id) REFERENCES public.zkapp_uris(id);


--
-- Name: zkapp_verification_keys zkapp_verification_keys_hash_id_fkey; Type: FK CONSTRAINT; Schema: public; Owner: postgres
--

ALTER TABLE ONLY public.zkapp_verification_keys
    ADD CONSTRAINT zkapp_verification_keys_hash_id_fkey FOREIGN KEY (hash_id) REFERENCES public.zkapp_verification_key_hashes(id);


--
-- PostgreSQL database dump complete
--

