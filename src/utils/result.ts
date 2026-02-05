export type ResultOk<T> = { data: T; error: null };
export type ResultError<E> = { data: null; error: E };

export type Result<T, E = Error> = ResultOk<T> | ResultError<E>;

export async function tryCatch<T, E = Error>(cb: Promise<T>): Promise<Result<T, E>> {
  try {
    const data = await cb;

    return { data: data as T, error: null };
  } catch (error: unknown) {
    return { data: null, error: error as E };
  }
}
