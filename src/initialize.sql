-- Copyright 2021 Matthew James Kraai
--
-- This file is part of odo.
--
-- odo is free software: you can redistribute it and/or modify it under the terms of the GNU Affero
-- General Public License as published by the Free Software Foundation, either version 3 of the
-- License, or (at your option) any later version.
--
-- odo is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the
-- implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU Affero
-- General Public License for more details.
--
-- You should have received a copy of the GNU Affero General Public License along with odo.  If not,
-- see <https://www.gnu.org/licenses/>.

CREATE TABLE IF NOT EXISTS actions (description PRIMARY KEY);
CREATE TABLE IF NOT EXISTS goals (description PRIMARY KEY, action TEXT REFERENCES actions (description) ON DELETE SET NULL ON UPDATE CASCADE);
